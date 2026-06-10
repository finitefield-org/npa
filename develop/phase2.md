These five items are the core of **Phase 2: certificate**. Phase 1 established
that the kernel can check core terms. Phase 2 goes one step further and creates
a recheckable artifact that is independent of source code, tactics, and AI
output.

The target flow is:

```text
source / tactic / AI / elaborator
  ↓
canonical core AST
  ↓
module certificate
  ↓
hashes + axiom report
  ↓
rechecked by the Phase 2 certificate verifier + Rust kernel
  ↓
the same .npcert is rechecked by the Phase 8 independent checker
```

---

# 1. Purpose of Phase 2

Phase 2 produces proof certificate files such as `.npcert`.

```text
Std/Nat/Basic.npa      -- source written by a human
Std/Nat/Basic.npcert   -- proof certificate read by the checker; the kernel checks decoded core declarations
```

The important point is that the checker reads the certificate, not the source.

```text
Not trusted:
  parser
  notation
  elaborator
  tactic
  AI
  source-level macro

Trusted:
  Phase 2 certificate verifier that rechecks the canonical certificate
  kernel that checks decoded core declarations
```

The Phase 2 certificate verifier is a decoder / verifier in the same
implementation stack that calls the Phase 1 Rust kernel. The Phase 8 independent
checker is later work that rechecks the same `.npcert` through another
implementation or process.

At minimum, the Phase 2 artifact contains:

```text
- canonical core AST
- module certificate
- import hash
- declaration hash
- axiom report
```

---

# 2. Canonical Core AST

## 2.1 Purpose

The canonical core AST is the standard kernel representation after all surface
syntax has been erased.

Surface syntax may write:

```text
theorem id (A : Type) (x : A) : A := x
```

Inside the certificate this is stored as a fully explicit term:

```text
Lam A : Sort u,
  Lam x : BVar 0,
    BVar 0
```

Canonical core AST requirements:

```text
- no notation
- no implicit arguments
- no typeclass placeholder
- no unresolved metavariable
- no tactic
- no macro
- no source-level match
- no dependency on binder names
- universe levels are explicit
- global constant references are unique
```

Therefore the following two terms have the same canonical AST:

```text
λ x : Nat, x
λ y : Nat, y
```

Binder names carry no meaning.

## 2.2 Core AST Shape

Phase 2 canonicalizes the Phase 1 term language.

```text
Term ::=
  Sort Level
| BVar Index
| Const GlobalRef [Level]
| App Term Term
| Lam Type Body
| Pi  Type Body
| Let Type Value Body
```

Binder names are not part of the certificate body. Source maps and display names
may exist in a separate debug area, but they are not used by kernel checking or
hashing.

## 2.3 Canonical Encoding

Conceptually, use tagged nodes:

```text
00 Sort(level)
01 BVar(index)
02 Const(global_ref, levels)
03 App(fn, arg)
04 Lam(type, body)
05 Pi(type, body)
06 Let(type, value, body)
```

For example:

```text
Π A : Sort u, A → A
```

is internally:

```text
Pi
  type = Sort u
  body =
    Pi
      type = BVar 0
      body = BVar 1
```

Because de Bruijn indices are used, the representation is independent of binder
names.

## 2.4 GlobalRef

A `Const` reference by name alone is unsafe.

Bad:

```text
Const "Nat.add"
```

Another import may provide the same name with different contents.

The canonical form is:

```text
Const {
  import_index,
  declaration_name,
  decl_interface_hash
}
```

For a declaration in the current module:

```text
Const {
  local_declaration_index
}
```

The reference fixes not only the name `Nat.add`, but also which import and which
declaration hash it refers to.

## 2.5 Term Hash

Each term has a hash:

```text
term_hash(t) = H("NPA-TERM-0.1" || canonical_encode(t))
```

`"NPA-TERM-0.1"` is a fixed domain-separation string. This avoids confusing
different hash roles even if raw digests collide across object kinds.

```text
H("NPA-TERM-0.1" || ...)
H("NPA-DECL-CERT-0.1" || ...)
H("NPA-MODULE-EXPORT-0.1" || ...)
```

---

# 3. Module Certificate

## 3.1 Purpose

A module certificate is the artifact used to check one whole module.

For example, the module:

```text
Std.Nat.Basic
```

produces:

```text
Std.Nat.Basic.npcert
```

It contains imports, declarations, hashes, and an axiom report.

## 3.2 Overview

Conceptually:

```json
{
  "format": "NPA-CERT-0.1",
  "core_spec": "NPA-Core-0.1",
  "module": "Std.Nat.Basic",
  "imports": [],
  "declarations": [],
  "export_block": [],
  "axiom_report": {
    "module_axioms": [],
    "per_declaration": []
  },
  "hashes": {
    "export_hash": "sha256:...",
    "axiom_report_hash": "sha256:...",
    "certificate_hash": "sha256:..."
  }
}
```

The actual storage format is canonical binary, not JSON. JSON is only for
explanation and debugging. The JSON above is a logical/debug view; the canonical
hash payload must not use maps. `export_block`, `axiom_report`, and `hashes` are
encoded as fixed-order records or length-prefixed arrays.

### 3.2.1 Minimum Requirements for Canonical Binary

Canonical binary follows the canonicalization / binary encoding requirements in
`core-spec-v0.1.md`. The Phase 2 implementation must at least satisfy:

```text
- unsigned integers are minimal ULEB128
- strings are byte length + UTF-8 bytes
- enum variants are fixed numeric tags
- record field order is fixed
- arrays have explicit length
- maps are forbidden in hash payloads
- optional fields have explicit 0/1 tags
- name table is in UTF-8 byte lexicographic order
- level / term DAGs are in topological order; ties use structural tag order
- imports are ordered lexicographically by module name, export_hash, and certificate_hash option/value
- declarations are in dependency order; ties use declaration-name UTF-8 byte lexicographic order
- sha256 is computed over the canonical byte sequence itself
```

Reject nonminimal ULEB128, unused term table entries, order violations, invalid
UTF-8, and maps inside hash payloads as `NonCanonicalEncoding`. Reject unknown
enum tags as `UnsupportedEncoding`. Source maps, diagnostics, AI traces, display
names, and other metadata are not part of the trusted payload or hash input.

The trusted v0.1 `TermNode` schema has no variant for metavariables, holes, or
placeholders. The certificate producer must reject unresolved metavariables
before emitting `.npcert`. A future pre-certificate API may return
`UnresolvedMetavariable`, but a metavariable-like unknown tag inside a v0.1
`.npcert` is `UnsupportedEncoding`.

Reasons for binary:

```text
- JSON canonicalization is fragile because of whitespace, order, and escaping
- binary is faster
- hash input is easier to fix
- term table / name table / DAG sharing is easier
```

## 3.3 ModuleCert Structure

Rust-style:

```rust
struct ModuleCert {
    header: CertHeader,
    imports: Vec<ImportEntry>,
    name_table: NameTable,
    level_table: LevelTable,
    term_table: TermTable,
    declarations: Vec<DeclCert>,
    export_block: ExportBlock,
    axiom_report: AxiomReport,
    hashes: ModuleHashes,
}
```

Roles:

```text
header:
  certificate format, core specification, module name, and similar data

imports:
  dependency modules and their hashes

name_table:
  canonical encoding of global names

level_table:
  shared universe levels

term_table:
  DAG representation of core terms

declarations:
  def/theorem/axiom/inductive declarations defined in this module

export_block:
  public interface consumed by downstream modules

axiom_report:
  list of axioms used

hashes:
  export_hash, axiom_report_hash, certificate_hash
```

## 3.4 Declaration Kinds

Phase 2 certificates contain at least these four declaration kinds:

```text
AxiomDecl
DefDecl
TheoremDecl
InductiveDecl
```

### AxiomDecl

```json
{
  "kind": "axiom",
  "name": "Classical.choice",
  "universe_params": ["u"],
  "type": "..."
}
```

`AxiomDecl` is inherently risky. High-trust mode uses an allowlist.

### DefDecl

```json
{
  "kind": "def",
  "name": "Nat.add",
  "universe_params": [],
  "type": "...",
  "value": "...",
  "reducibility": "reducible"
}
```

The kernel checks:

```text
value : type
```

If the definition is `reducible`, it participates in δ-reduction.

### TheoremDecl

```json
{
  "kind": "theorem",
  "name": "Nat.add_zero",
  "universe_params": [],
  "type": "Π n : Nat, Eq Nat (Nat.add n Nat.zero) n",
  "proof": "...",
  "opacity": "opaque"
}
```

The kernel checks:

```text
proof : type
```

Theorems are normally opaque, so the conversion checker does not unfold proofs.

### InductiveDecl

```json
{
  "kind": "inductive",
  "name": "Nat",
  "universe_params": [],
  "params": [],
  "indices": [],
  "sort": "Sort 1",
  "constructors": [
    {
      "name": "Nat.zero",
      "type": "Nat"
    },
    {
      "name": "Nat.succ",
      "type": "Nat → Nat"
    }
  ]
}
```

The kernel checks:

```text
- constructor types are correct
- return type is the target inductive
- strict positivity holds
- universe constraints are consistent
- the recursor can be generated correctly
```

---

# 4. Import Hash

## 4.1 Why It Is Needed

Managing imports by name alone is unsafe.

```text
import Std.Nat.Basic
```

This does not prove that the loaded `Std.Nat.Basic` is the expected one.

Therefore imports carry hashes.

```json
{
  "module": "Std.Nat.Basic",
  "export_hash": "sha256:...",
  "certificate_hash": "sha256:..."
}
```

`export_hash` is always required. `certificate_hash` is optional in normal mode
and required in high-trust mode.

## 4.2 Split export_hash and certificate_hash

This distinction is important.

Module certificates carry at least:

```text
export_hash:
  hash of public information needed by downstream type checking and conversion

certificate_hash:
  hash of the trusted payload including proof bodies, excluding certificate_hash itself

axiom_report_hash:
  hash of the canonical axiom report
```

The split is needed because opaque theorem proof bodies are unnecessary for
downstream type checking.

For example:

```text
theorem T : P := proof1
```

may change to:

```text
theorem T : P := proof2
```

If `T` is opaque, downstream modules see the same type. Audit still needs to
notice that the proof changed.

So:

```text
export_hash:
  what affects downstream type checking

certificate_hash:
  trusted payload including proofs, excluding certificate_hash itself
```

Formally, `export_hash` is:

```text
H("NPA-MODULE-EXPORT-0.1" || export_block)
```

over only the canonical export block. `certificate_hash` is:

```text
H("NPA-MODULE-CERT-0.1" || trusted_payload_without_certificate_hash)
```

Debug metadata, source maps, diagnostics, and AI traces are in neither hash.

## 4.3 What export_hash Includes

`export_hash` includes:

```text
- exported declaration interface hashes
- exported inductive declarations
- body hash of transparent/reducible definitions
- type hash of opaque theorems
- per-exported-declaration axiom dependency summary
- universe declarations
```

The hash input for `export_hash` is only `ExportBlockBytes(export_block)` from
section 11.11. Module name, certificate format version, and core spec version
are included on the `certificate_hash` side in
`ModuleCertBytesWithoutCertificateHash`.

Important:

```text
the body of a transparent def must be included in export_hash
```

because transparent definitions unfold during δ-reduction and affect downstream
conversion.

By contrast:

```text
the proof body of an opaque theorem need not be included in export_hash
```

but the theorem's axiom dependencies are included.

## 4.4 ImportEntry

```rust
struct ImportEntry {
    module_name: ModuleName,
    export_hash: Hash,
    certificate_hash: Option<Hash>,
}
```

Like `Import` in the core spec, the canonical import entry contains only module
name, `export_hash`, and optional `certificate_hash`.

High-trust mode also requires `certificate_hash`.

```text
normal mode:
  require export_hash match

high-trust mode:
  require export_hash match
  require certificate_hash match
  require the imported certificate to have already been checked
```

The visible declaration list from an import is not stored redundantly in the
import entry. The checker builds the environment from the `export_block` of a
checked import certificate and compares each `Const` / dependency entry against
its `decl_interface_hash`. Any implementation cache similar to `imported_decls`
is derived data and is not part of the canonical payload or hash input.

---

# 5. Declaration Hash

## 5.1 One Declaration Hash Is Not Enough

Declarations carry at least two hashes.

```text
decl_interface_hash:
  hash of the meaning visible downstream

decl_certificate_hash:
  hash of the whole declaration including proof/value body
```

Implementation helpers may also keep:

```text
type_hash
value_hash
proof_hash
```

## 5.2 DefDecl Hash

For transparent/reducible definitions, the body affects downstream conversion.

```text
decl_interface_hash(def)
  = H(
      "NPA-DECL-IFACE-0.1",
      kind = def,
      name,
      universe_params,
      type_hash,
      reducibility,
      public_dependency_entries,
      axiom_dependencies,
      value_hash if reducibility = reducible
    )
```

```text
decl_certificate_hash(def)
  = H(
      "NPA-DECL-CERT-0.1",
      decl_interface_hash,
      value_hash,
      dependency_entries,
      axiom_dependencies
    )
```

For a reducible `DefDecl`, `value_hash` is part of the interface. For an opaque
def, `value_hash` is included only in `decl_certificate_hash`. `DependencyEntry`
values appearing in public type / reducible body are also included in the
interface so changes to referenced `decl_interface_hash` values propagate into
downstream `export_hash`. Non-axiom dependencies from an opaque def body are not
public interface.

## 5.3 TheoremDecl Hash

Opaque theorem proofs are not used by downstream conversion.

```text
decl_interface_hash(theorem)
  = H(
      "NPA-DECL-IFACE-0.1",
      kind = theorem,
      name,
      universe_params,
      type_hash,
      opacity = opaque,
      public_dependency_entries,
      axiom_dependencies
    )
```

```text
decl_certificate_hash(theorem)
  = H(
      "NPA-DECL-CERT-0.1",
      decl_interface_hash,
      proof_hash,
      dependency_entries
    )
```

With this design, changing only the proof does not change
`decl_interface_hash`, but does change `decl_certificate_hash`.

If a proof change changes the axioms used, `axiom_dependencies` changes and the
interface hash also changes. This is intentional.

```text
theorem T : P := constructive_proof
```

to:

```text
theorem T : P := proof_using_Classical_choice
```

changes downstream trust, so the interface hash changes.

## 5.4 AxiomDecl Hash

An `AxiomDecl` is itself an axiom dependency.

```text
decl_interface_hash(axiom)
  = H(
      "NPA-DECL-IFACE-0.1",
      kind = axiom,
      name,
      universe_params,
      type_hash,
      public_dependency_entries
    )
```

`public_dependency_entries` is derived from direct `Const` references in the
axiom type, matching the `DeclInterfacePayload` rule in section 11.11.

In the axiom report:

```text
axiom_dependencies(axiom) = { axiom.name }
```

## 5.5 InductiveDecl Hash

Inductive declarations affect constructors and recursor computation rules.

```text
decl_interface_hash(inductive)
  = H(
      "NPA-DECL-IFACE-0.1",
      kind = inductive,
      name,
      universe_params,
      params,
      indices,
      sort,
      constructors,
      generated_recursor_signature_hash,
      generated_computation_rule_hash,
      public_dependency_entries,
      axiom_dependencies
    )
```

The whole inductive structure is part of the interface because it affects
downstream type checking and ι-reduction. `constructors` directly contains each
constructor name and constructor type hash. The recursor signature and
computation rule are not expanded inline; they are separated into dedicated
generated artifact hashes.

---

# 6. Axiom Report

## 6.1 Purpose

The axiom report makes explicit which axioms each theorem or module depends on.

In the canonical payload, axioms use the canonical order of `AxiomRef` described
later, and per-declaration reports are stored in declaration order. Explanatory
JSON may display maps keyed by declaration name, but hash payloads must not use
maps.

Example:

```json
{
  "module": "Std.Nat.Basic",
  "axioms_used": [],
  "declarations": {
    "Nat.add_zero": {
      "axioms_used": []
    }
  }
}
```

With classical logic:

```json
{
  "declarations": {
    "Classical.some_theorem": {
      "axioms_used": [
        "Classical.choice",
        "Propext"
      ]
    }
  }
}
```

## 6.2 Treat sorry as an Axiom

If `sorry` or `admit` is allowed, it is represented internally as an axiom.

```text
sorry : P
```

is effectively:

```text
axiom sorry_123 : P
```

It must therefore appear in the axiom report.

```json
{
  "axioms_used": [
    {
      "name": "synthetic.sorry.Std.Nat.Basic.add_zero",
      "kind": "sorry",
      "allowed": false
    }
  ]
}
```

High-trust mode fails immediately on this.

## 6.3 Per-Declaration Report

Record axiom dependencies for each declaration.

```json
{
  "name": "Nat.add_zero",
  "kind": "theorem",
  "direct_axioms": [],
  "transitive_axioms": [],
  "status": "constructive"
}
```

With a classical axiom:

```json
{
  "name": "Classical.choice_example",
  "kind": "theorem",
  "direct_axioms": ["Classical.choice"],
  "transitive_axioms": ["Classical.choice"],
  "status": "uses_allowed_axioms"
}
```

## 6.4 Computing Axiom Dependencies

Process declarations in dependency order.

```text
axioms(AxiomDecl a)
  = {a}

axioms(DefDecl d)
  = direct_axioms(type(d))
    ∪ direct_axioms(value(d))
    ∪ ⋃ axioms(dep) for dep in dependencies(d)

axioms(TheoremDecl t)
  = direct_axioms(type(t))
    ∪ direct_axioms(proof(t))
    ∪ ⋃ axioms(dep) for dep in dependencies(t)

axioms(InductiveDecl I)
  = direct_axioms(types in declaration)
    ∪ ⋃ axioms(dep) for dep in dependencies(I)
```

`dependencies(d)` are the `Const` references appearing in type, value, or proof.

## 6.5 Module-Level Report

For the whole module, union the axiom sets of each declaration.

```json
{
  "module": "Std.Nat.Basic",
  "module_axioms": [],
  "per_declaration": [],
  "custom_axioms": [],
  "standard_axioms": [],
  "contains_sorry": false,
  "safe_for_high_trust": true
}
```

`custom_axioms`, `standard_axioms`, `contains_sorry`, and
`safe_for_high_trust` are audit/policy views. The checker does not trust stored
booleans; it recomputes them from canonical `module_axioms` and the policy.

High-trust decision:

```text
safe_for_high_trust =
  contains_sorry == false
  && custom_axioms == []
  && all standard_axioms are allowlisted
```

---

# 7. Certificate Generation Pipeline

Phase 2 implements this pipeline:

```text
1. receive core declarations
2. check each declaration with the kernel
3. convert to canonical core AST
4. build name/level/term tables in canonical order
5. compute term hashes
6. compute declaration hashes
7. build per-declaration dependency entries
8. build the axiom report
9. build the export block
10. compute export_hash / certificate_hash / axiom_report_hash
11. write .npcert
12. checker decodes .npcert and imported .npcert files; the kernel rechecks decoded declarations
```

Pseudocode:

```rust
fn build_certificate(module: CoreModule, imports: Vec<VerifiedImport>) -> Result<ModuleCert> {
    let mut env = Env::from_imports(&imports);
    let mut decl_certs = Vec::new();

    for decl in module.declarations {
        check_declaration(&env, &decl)?;

        let canonical = canonicalize_decl(&decl)?;
        let deps = collect_dependencies(&canonical);
        let axiom_deps = compute_axiom_deps(&env, &deps, &canonical);

        let hashes = compute_decl_hashes(&canonical, &deps, &axiom_deps);

        let cert = DeclCert {
            canonical,
            deps,
            axiom_deps,
            hashes,
        };

        env.add_decl_interface(&cert)?;
        decl_certs.push(cert);
    }

    let export_block = build_export_block(&decl_certs);
    let axiom_report = build_axiom_report(&decl_certs);
    let tables = build_canonical_tables(&decl_certs)?;
    let hashes = compute_certificate_hashes(&imports, &tables, &export_block, &decl_certs, &axiom_report);

    Ok(ModuleCert {
        header: make_header(),
        imports,
        name_table: tables.names,
        level_table: tables.levels,
        term_table: tables.terms,
        declarations: decl_certs,
        export_block,
        axiom_report,
        hashes,
    })
}
```

## 7.1 Producer Separation Policy

The Phase 2 trusted boundary does not change when producers are separated. A
producer is an untrusted layer that creates a `CoreModule` or core declaration
candidate before `.npcert` is produced.

```text
Human producer:
  human source / notation / display name / source map
  ↓
  CoreModule

AI producer:
  structured AI request / explicit core-like term / batch candidate
  ↓
  CoreModule or CoreDeclCandidate

Common path:
  CoreModule
  ↓
  npa-cert build / verify
  ↓
  npa-kernel check
```

`npa-cert` and `npa-kernel` do not trust the producer kind. Human or AI origin is
not part of the trusted payload. The same `CoreModule` and same import set must
produce the same `.npcert` bytes and hashes.

Not included in trusted payload:

```text
producer kind
source text
source map
pretty/display name
elaborator trace
tactic trace
AI prompt / completion / score / trace
model name
search rank
cache hit / cache miss
```

Producer metadata belongs in a debug sidecar / audit sidecar outside `.npcert`.
Changing sidecars must not change `term_hash`, `decl_interface_hash`,
`export_hash`, `certificate_hash`, or `axiom_report_hash`.

### 7.1.1 Human Producer

The Human producer creates `CoreModule` from the Phase 3+ human surface language.
It may prioritize readability.

Untrusted inputs allowed:

```text
source text
namespace / open
notation
implicit arguments
holes before elaboration
display binder name
source location
diagnostics
```

Immediately before handing data to `.npcert`, it must ensure:

```text
- no unresolved metavariable / hole remains
- no notation / implicit argument / typeclass placeholder remains
- terms can be converted to a binder-name-independent de Bruijn representation
- universe levels are explicit
- global constant references are fixed by import hash and decl_interface_hash
```

Source maps and display names are sidecars, not inputs to kernel checks,
certificate hashes, or export hashes.

### 7.1.2 AI Producer

The AI producer tries many candidates quickly and does not go through the human
surface language. Its input should be as close as possible to canonical core.

The AI producer MVP does not use:

```text
- notation
- open / namespace dependent short names
- overload resolution
- implicit argument insertion
- unresolved hole
- tactic script text
- source-level axiom declaration
- source-level inductive syntax
- typeclass search
- numeric literal overload
```

It may generate only:

```text
- fully qualified global reference as lookup input only
- explicit universe application
- Sort / Pi / Lam / App / Let / Const / BVar
- core def / theorem declarations
- GlobalRef entries present in verified import export blocks
- Local refs to prior declarations in the current module
```

Fully qualified global references are only lookup input. A `Const` stored in a
`CoreDeclCandidate` must be a Phase 2 hash-bound payload such as:

```text
GlobalRef::Imported(import_index, name, decl_interface_hash)
GlobalRef::Local(decl_index)
GlobalRef::LocalGenerated(decl_index, name)
GlobalRef::Builtin(name, decl_interface_hash)
```

Pretty names or fully qualified names alone are not certificate-facing core
terms.

AI producer output is not a trusted certificate. `npa-cert` receives it as
`CoreDeclCandidate` or `CoreModule` and still performs canonicalization, hash
recomputation, axiom dependency computation, and kernel checking.

### 7.1.3 AI Candidate Fast Path

Producing a full `.npcert` for every AI candidate is expensive, so Phase 2 may
provide a certificate-free candidate fast path.

```text
AI candidate batch
  ↓
schema / size limit check
  ↓
import ref validation
  ↓
kernel precheck
  ↓
candidate accepted?
  ↓ yes
CoreDeclCandidate / CheckedDeclCandidate
  ↓
only accepted candidates reach build_module_cert
```

Fast-path success only means that the candidate passed kernel precheck in the
current verified import environment. It is not a proved module. To become a
final artifact, it must still go through `build_module_cert` and
`verify_module_cert`.

The fast path must not skip:

```text
- core AST schema validation
- rejection of unresolved metavariables / placeholders
- decl_interface_hash matching for imported GlobalRef values
- well-scoped declaration dependencies
- kernel check of proof : theorem_type / value : def_type
- universe level well-formedness
```

It may skip only artifact-production work:

```text
may skip:
  .npcert byte emission
  module certificate_hash computation
  final export_block construction
  sidecar generation

must not skip:
  kernel check
  import interface check
  unresolved hole rejection
  resource limit enforcement
```

### 7.1.4 Batch / Cache

The AI producer may use a batch API. A batch checks candidates against the same
import environment and same prior declarations.

```text
Batch input:
  verified imports
  checked prior current declarations
  candidates[]
  deterministic budget
  resource limit

Batch output:
  per-candidate success / structured error
  optional term_hash / decl hash preview
  optional normalized core size
  no trusted certificate
```

Performance caches may include:

```text
- import kernel environment cache keyed by import module + export_hash
- import certificate / high-trust verification cache keyed by import module + export_hash + certificate_hash
- name / level / term hash-consing cache
- WHNF cache
- conversion cache
- declaration dependency cache
```

Caches are not evidence for correctness. Cache hit/miss is not part of trusted
payloads, hashes, axiom reports, or certificate identity. Disabling caches must
not change success/failure results or certificate bytes for the same input.

### 7.1.5 Producer Sidecar

Producers may emit sidecars for debugging, audit, or training.

```rust
struct ProducerSidecar {
    module: ModuleName,
    producer_profile: ProducerProfile,
    producer_run_id: String,
    candidate_count: u64,
    accepted_candidate_count: u64,
    diagnostics: Vec<ProducerDiagnostic>,
    input_artifact_hashes: Vec<Hash>,
}

enum ProducerProfile {
    HumanSurface,
    AiCoreMvp,
}
```

This sidecar is separate from `.npcert`. It is not part of
`ModuleCertBytes`. The Phase 2 verifier must be able to check `.npcert` without
reading any sidecar.

### 7.1.6 Forbidden Shortcuts

Producer separation forbids these shortcuts:

```text
- adopting hashes computed by the AI producer as trusted hashes
- skipping kernel checks for declarations the AI producer labels as `verified`
- treating candidate fast-path success as `.npcert` verification success
- reducing verifier checks based on producer kind
- completing GlobalRef from pretty name / source span / model score
- allowing the producer to implicitly complete the import store from filesystem / network
- representing unresolved metavariables in the certificate schema
```

Producers are separated for speed. The source of truth is always the canonical
certificate.

---

# 8. Certificate Verification Pipeline

The Phase 2 checker is the certificate verifier. It decodes canonical binary,
recomputes hashes, axiom reports, and imports, and passes decoded declarations to
the Phase 1 Rust kernel. It does not inspect source, source maps, elaborator
traces, tactic traces, or AI traces.

This verifier is not the Phase 8 independent checker. Phase 8 consumes the
Phase 2 `.npcert` format and performs the same checks through a separate
implementation or process.

```text
.npcert
  ↓
parse canonical binary
  ↓
confirm import export_hash, and certificate_hash in high-trust mode
  ↓
recompute declaration hashes
  ↓
kernel check
  ↓
recompute axiom report
  ↓
recompute export_hash / certificate_hash / axiom_report_hash
  ↓
accepted
```

Pseudocode:

```rust
fn check_certificate(cert: ModuleCert, import_store: ImportStore) -> Result<VerifiedModule> {
    verify_header(&cert)?;

    let imports = load_and_check_imports(&cert.imports, import_store)?;
    let mut env = Env::from_imports(&imports);

    verify_canonical_tables(&cert.name_table, &cert.level_table, &cert.term_table)?;

    for decl in &cert.declarations {
        verify_canonical_encoding(decl)?;
        verify_decl_hashes(decl)?;

        check_declaration(&env, &decl.canonical)?;

        let deps = collect_dependencies(&decl.canonical);
        let ax = compute_axiom_deps(&env, &deps, &decl.canonical);

        if ax != decl.axiom_deps {
            return Err(Error::AxiomReportMismatch);
        }

        env.add_decl_interface(decl)?;
    }

    verify_module_axiom_report(&cert)?;
    verify_axiom_report_hash(&cert)?;
    verify_export_hash(&cert)?;
    verify_module_certificate_hash(&cert)?;

    Ok(VerifiedModule::new(cert))
}
```

---

# 9. Minimal Certificate Example

Consider a module containing only `id`.

```text
def id.{u} : Π A : Sort u, A → A :=
  λ A : Sort u, λ x : A, x
```

Conceptual certificate:

```json
{
  "format": "NPA-CERT-0.1",
  "core_spec": "NPA-Core-0.1",
  "module": "Std.Logic.Id",
  "imports": [],
  "declarations": [
    {
      "kind": "def",
      "name": "id",
      "universe_params": ["u"],
      "type": {
        "core": "Pi (Sort u) (Pi (BVar 0) (BVar 1))",
        "hash": "term:..."
      },
      "value": {
        "core": "Lam (Sort u) (Lam (BVar 0) (BVar 0))",
        "hash": "term:..."
      },
      "reducibility": "reducible",
      "decl_interface_hash": "decl-iface:...",
      "decl_certificate_hash": "decl-cert:...",
      "axioms_used": []
    }
  ],
  "export_block": [
    {
      "name": "id",
      "decl_interface_hash": "decl-iface:..."
    }
  ],
  "axiom_report": {
    "module_axioms": [],
    "per_declaration": [
      {
        "decl": "id",
        "axioms": []
      }
    ]
  },
  "hashes": {
    "export_hash": "sha256:...",
    "axiom_report_hash": "sha256:...",
    "certificate_hash": "sha256:..."
  }
}
```

In the real certificate, `core` is not a string; it is a canonical binary term
table.

---

# 10. Error Conditions

The Phase 2 checker fails when:

```text
- import export_hash, or certificate_hash in high-trust mode, does not match
- certificate format version is unsupported
- core AST is not canonical
- unresolved metavariable remains
- declaration hash differs from recomputation
- term hash differs from recomputation
- proof : theorem_type does not hold
- def value : def_type does not hold
- inductive declaration fails positivity
- axiom report differs from recomputation
- a policy-forbidden axiom is present
- sorry is present when policy has deny_sorry
- export_hash differs from recomputation
- certificate_hash differs from recomputation
```

---

# 11. Phase 2 Implementation Contract

This chapter is the implementation contract for Phase 2. It turns the preceding
design into concrete rules. Type checking, conversion, and inductive logical
rules not specified here follow `core-spec-v0.1.md` and the Phase 1 kernel
specification.

## 11.1 Crate / API Boundary

Certificate processing is separated from the kernel.

```text
crates/npa-kernel:
  core Expr / Level / Decl
  type checking
  definitional equality
  reduction
  inductive checking
  no file I/O
  no serialization
  no hashing
  no policy decision

crates/npa-cert:
  canonicalization
  canonical binary encode/decode
  hash calculation
  axiom dependency calculation
  import store / high-trust policy
  certificate build / verify
  calls npa-kernel with decoded declarations
```

Minimum public API:

```rust
pub type Hash = [u8; 32];

pub struct Name(pub Vec<String>);

pub type ModuleName = Name;
pub type AxiomName = Name;

pub struct CoreModule {
    pub name: ModuleName,
    pub declarations: Vec<npa_kernel::Decl>,
}

pub enum TrustMode {
    Normal,
    HighTrust,
}

pub struct AxiomPolicy {
    pub mode: TrustMode,
    pub allowlisted_axioms: BTreeSet<AxiomName>,
    pub deny_sorry: bool,
}

pub struct VerifierSession {
    checked: BTreeMap<ImportKey, VerifiedModule>,
}

pub fn build_module_cert(
    module: CoreModule,
    imports: &[VerifiedModule],
) -> Result<ModuleCert, CertError>;

pub fn encode_module_cert(cert: &ModuleCert) -> Result<Vec<u8>, CertError>;

pub fn decode_module_cert(bytes: &[u8]) -> Result<ModuleCert, CertError>;

pub fn verify_module_cert(
    bytes: &[u8],
    session: &mut VerifierSession,
    policy: &AxiomPolicy,
) -> Result<VerifiedModule, CertError>;
```

`Name` follows this grammar at the certificate / kernel boundary.

```text
DeclarationName = Component ("." Component)*
Component       = [A-Za-z_][A-Za-z0-9_']*
```

Only ASCII apostrophe (`U+0027`) is allowed. Unicode prime-like characters are
not canonical names. Operator symbols such as `+` and `*` are surface notation
and are not trusted certificate declaration names.

`verify_module_cert` performs decode, canonical encoding check, hash
recomputation, import resolution, axiom policy, and kernel check. `decode_module_cert`
is only syntactic decode and does not produce a trusted module.

`npa-kernel` does not read paths, filesystem, network, clocks, randomness, or
environment variables. Canonical byte generation must not depend on `HashMap` /
`HashSet` iteration order. Use `BTreeMap`, `BTreeSet`, or explicitly sorted
`Vec` when order matters.

### 11.1.1 Producer API Boundary

Human and AI producers live outside `npa-cert`. `npa-cert` must not accept
producer-specific inputs as trusted artifacts. `ProducerProfile` is sidecar /
audit classification and must not be an argument to certificate build / verify
APIs.

```rust
// sidecar / audit only
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

pub struct CheckedDeclCandidate {
    // private fields; construct only through check_core_decl_candidates
    declaration: npa_kernel::Decl,
    preview_hashes: CandidateHashPreview,
    pre_env_fingerprint: Hash,
    post_env_fingerprint: Hash,
    prior_chain_fingerprint: Hash,
    limits: ProducerLimits,
    limit_profile_hash: Hash,
    decl_interface_hash: Hash,
    decl_certificate_hash: Hash,
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
    // one status per input candidate, in the same order
    pub statuses: Vec<CandidateStatus>,
}

pub fn check_core_decl_candidates(
    batch: CandidateBatch<'_>,
) -> Result<CandidateBatchResult, CertError>;

pub fn build_module_cert_from_checked_candidates(
    module_name: ModuleName,
    imports: &[VerifiedModule],
    checked_decls: &[CheckedDeclCandidate],
) -> Result<ModuleCert, CertError>;
```

`check_core_decl_candidates` is the fast-path API for AI search. It does not
generate `.npcert` and does not return `VerifiedModule`. `Accepted` means only
that the candidate passed schema validation and kernel precheck under the given
import/prior declaration environment and limits.

`CandidateBatch.imports` is passed in the same canonical import order as
`ModuleCert.Imports`. `GlobalRef::Imported(import_index, ...)` in a
`CoreDeclCandidate` refers to `batch.imports[import_index]`. Reject the whole
batch with `Err(CertError)` if import order is noncanonical or if
`ProducerImportEnvKey(module, export_hash)` is duplicated. Different certificate
hashes for the same module / export_hash cannot appear simultaneously as direct
imports, because they are the same producer public environment.

`Err(CertError)` is for batch-level failures that cannot be assigned to a single
candidate, such as invalid prior tokens, invalid batch schema, or inconsistent
limit profiles. On `Ok(result)`, `result.statuses.len() == batch.candidates.len()`,
and `statuses[i]` is the result for `batch.candidates[i]`. Do not reorder by
score, hash, success/failure, or cache state.

`CheckedDeclCandidate` is an opaque checked token. Callers must not be able to
construct it directly from arbitrary `npa_kernel::Decl`. A prior token may be
added to the environment only if all of these hold:

```text
- the first token pre_env_fingerprint matches the initial env fingerprint recomputed from batch.imports
- each later token pre_env_fingerprint matches the previous token post_env_fingerprint
- token prior_chain_fingerprint matches the accepted prior declarations before it
- private decl_interface_hash / decl_certificate_hash match recomputation from the declaration
- the declaration was kernel-prechecked under the same limit profile or a stricter deterministic profile
- post_env_fingerprint matches recomputation after adding the token declaration interface to the producer public environment
```

If a prior token fails validation, reject the whole batch with a deterministic
structured error. Any API that accepts unchecked raw prior declarations must
recheck them from the beginning and convert them to `CheckedDeclCandidate`
tokens before checking later candidates.

`ProducerLimits` has canonical bytes: fields in struct order, each encoded as
minimal ULEB128.

```text
producer_limits_hash(limits) =
  sha256("NPA-PRODUCER-LIMITS-0.1" || canonical_encode(limits))
```

`CheckedDeclCandidate.limit_profile_hash` is this hash. A prior token can be
reused with current `batch.limits` only if the token was produced under the same
limits or stricter limits. Strictness compares all upper bounds:

```text
stricter_or_equal(a, b) =
  a.max_declarations      <= b.max_declarations
  && a.max_expr_nodes     <= b.max_expr_nodes
  && a.max_level_nodes    <= b.max_level_nodes
  && a.max_name_components <= b.max_name_components
  && a.max_reduction_steps <= b.max_reduction_steps
  && a.max_conversion_steps <= b.max_conversion_steps
```

Here `a` is the token's limits and `b` is the current batch limits. Store the
original `ProducerLimits` in the token's private `limits` field. Do not infer
strictness from hashes alone.

Producer token environment / chain fingerprints are Phase 2 concepts. Do not
confuse them with Phase 4 / Phase 5 proof-state fingerprints. Use fixed record
order and these domain separators:

```text
producer_env_fingerprint(env) =
  sha256("NPA-PRODUCER-ENV-0.1" || ProducerEnvFingerprintBytes(env))

ProducerEnvFingerprintBytes(env):
  direct_imports: vec<ProducerImportEnvKey>      -- canonical import order
  checked_decls: vec<ProducerCheckedDeclInterface>

ProducerImportEnvKey:
  module: ModuleName
  export_hash: hash

ProducerCheckedDeclInterface:
  decl_interface_hash: hash
  axiom_dependencies: vec<AxiomRef>   -- canonical order
```

`producer_env_fingerprint` is not a pure kernel-environment fingerprint. It is
the producer public environment used for AI reuse: import-reconstructed kernel
environment, current-module public declaration interfaces, and axiom trust
summary that affects downstream trust.

`ProducerImportEnvKey` intentionally excludes `certificate_hash`, because
changing only the proof body of an imported module while preserving
`export_hash` does not change the downstream kernel environment. Exact import
identity and high-trust verification are handled by `ImportEntry.certificate_hash`,
`VerifiedModule.certificate_hash`, and the high-trust verification cache.

`ProducerCheckedDeclInterface` does not store declaration name separately; it is
part of `decl_interface_hash`. It also excludes `decl_certificate_hash` because
opaque proof/body-only changes that preserve public interface and axiom
dependencies do not change the producer public environment. Exact token sequence
identity is fixed by the private token hash and by
`ProducerPriorChainEntry.decl_certificate_hash`.

Duplicate names, declaration order, and module-level visibility are finally
checked by `build_module_cert` over the whole `CoreModule`. Producer environment
fingerprints are auxiliary hashes for token-chain environment identity, not a
replacement for module validity checks.

`ProducerCheckedDeclInterface.axiom_dependencies` uses the same computation
rules as certificate generation. Separate the canonical bytes used for
fingerprinting from the operational lookup environment used by dependency /
axiom lookup.

```text
ProducerLookupEnv:
  import_exports: vec<ExportBlockView>          -- same canonical import order as batch.imports
  checked_decls: vec<ProducerCheckedDeclInterface>

producer_lookup_env(imports, checked_decls):
  import_exports = canonical_import_export_views(imports)
  checked_decls = checked_decls

producer_checked_decl_interface(decl, lookup_env):
  canonical = canonicalize_decl(decl)
  deps = collect_dependencies(canonical)
  axiom_dependencies = compute_axiom_deps(lookup_env, deps, canonical)
  hashes = compute_decl_hashes(canonical, deps, axiom_dependencies)
  return {
    decl_interface_hash = hashes.decl_interface_hash,
    axiom_dependencies = canonical_order(axiom_dependencies)
  }
```

`ProducerEnvFingerprintBytes` and `ProducerLookupEnv` represent the same
producer public environment for different purposes. The first is for hashing;
the second is for `compute_axiom_deps` / dependency lookup. The canonical import
order used by `canonical_import_env_keys(imports)` and
`canonical_import_export_views(imports)` must be identical. A
`GlobalRef::Imported(import_index, ...)` points simultaneously to
`imports[import_index]`, `direct_imports[import_index]`, and
`import_exports[import_index]`. Do not use only
`ProducerImportEnvKey(module, export_hash)` to look up axiom dependencies inside
an import. Do not trust dependency reports or preview hashes provided by the AI
producer.

The initial producer public environment contains only imports:

```text
initial_env_fingerprint(imports) =
  producer_env_fingerprint({
    direct_imports = canonical_import_env_keys(imports),
    checked_decls = []
  })
```

After adding a declaration, recompute the producer public environment
fingerprint from the whole import and checked-declaration interface sequence.
Do not reuse previous environment bytes, so implementation-specific incremental
caches cannot affect the fingerprint.

```text
post_env_fingerprint(imports, checked_decls_before, decl) =
  pre_env_bytes = {
    direct_imports = canonical_import_env_keys(imports),
    checked_decls = checked_decls_before
  }
  lookup_env = producer_lookup_env(imports, checked_decls_before)
  producer_env_fingerprint({
    direct_imports = pre_env_bytes.direct_imports,
    checked_decls = pre_env_bytes.checked_decls ++ [producer_checked_decl_interface(decl, lookup_env)]
  })
```

The prior chain fingerprint fixes the order of checked declarations in the
current module. Imports are already in the environment fingerprint and are not
included here.

```text
prior_chain_fingerprint(chain) =
  sha256("NPA-PRODUCER-CHAIN-0.1" || ProducerPriorChainBytes(chain))

ProducerPriorChainBytes(chain):
  checked_decls: vec<ProducerPriorChainEntry>

ProducerPriorChainEntry:
  decl_interface_hash: hash
  decl_certificate_hash: hash
  pre_env_fingerprint: hash
  post_env_fingerprint: hash
```

The first token's `prior_chain_fingerprint` is computed from the empty chain.
Later tokens must match recomputation from the preceding accepted prior
declarations.

`preview_hashes` may help logging, dedupe, and ranking, but not token
validation. Each `CandidateHashPreview` field is optional and non-authoritative.
If a preview hash differs from the accepted token's private
`decl_interface_hash` / `decl_certificate_hash`, report a diagnostic if useful,
but do not adopt the preview. Final `decl_interface_hash`,
`decl_certificate_hash`, `export_hash`, and `certificate_hash` are trusted only
when recomputed by `build_module_cert` and `verify_module_cert`.

`build_module_cert_from_checked_candidates` builds the final `ModuleCert` from
accepted tokens only. It revalidates each token's `pre_env_fingerprint`,
`post_env_fingerprint`, `prior_chain_fingerprint`,
`producer_limits_hash(token.limits) == token.limit_profile_hash`, and private
declaration hashes. Only if the token chain exactly matches the `imports` and
`checked_decls` order does it internally construct `CoreModule` and call
`build_module_cert`. Do not expose a public getter that lets the caller assemble
a raw `CoreModule` from token declarations. This API does not perform strictness
comparison against new `ProducerLimits`; strictness is only checked when
`check_core_decl_candidates` reuses prior tokens under current `batch.limits`.

## 11.2 Canonical Binary Primitive

Every payload uses only these primitives:

```text
uvar:
  minimal unsigned LEB128
  non-minimal form is NonCanonicalEncoding

tag:
  exactly one byte
  unknown tag is UnsupportedEncoding

bytes:
  uvar byte_length + raw bytes

string:
  same as bytes
  content must be valid UTF-8
  invalid UTF-8 is NonCanonicalEncoding

vec<T>:
  uvar length + element bytes in order

option<T>:
  tag 0x00 for none
  tag 0x01 + T for some

hash:
  raw 32-byte sha256 digest
  "sha256:..." hex string is debug view only

record:
  fields concatenated in the order specified below
  field names are not encoded
```

The same value always encodes to the same bytes. If reencoding after decode does
not reproduce the input bytes, reject as `NonCanonicalEncoding`.

## 11.3 Canonical Name / ID

Names inside canonical payloads are component lists, not dotted strings.

```text
Name:
  vec<string component>

NameId:
  uvar index into name_table
```

Name ordering compares components left-to-right by UTF-8 byte lexicographic
order. If one name is a prefix of the other, the shorter one comes first.

`Name` is a non-empty component list. Components must not be empty and must not
contain `.`. `name_table` has no duplicates and is sorted by this order. Display
names and source binder names are not trusted payload.

## 11.4 Module Certificate Byte Schema

The trusted payload of an on-disk `.npcert` is encoded in this order:

```text
ModuleCertBytes =
  Header
  Imports
  NameTable
  LevelTable
  TermTable
  Declarations
  ExportBlock
  AxiomReport
  ModuleHashes

Header =
  format: string              -- "NPA-CERT-0.1"
  core_spec: string           -- "NPA-Core-0.1"
  module: Name

Imports =
  vec<ImportEntry>

ImportEntry =
  module: Name
  export_hash: hash
  certificate_hash: option<hash>

NameTable =
  vec<Name>

LevelTable =
  vec<LevelNode>

TermTable =
  vec<TermNode>

Declarations =
  vec<DeclCert>

ModuleHashes =
  export_hash: hash
  axiom_report_hash: hash
  certificate_hash: hash
```

For `certificate_hash` computation only, replace the final `ModuleHashes` with:

```text
ModuleHashesForCertificateHash =
  export_hash: hash
  axiom_report_hash: hash
```

So `certificate_hash` includes neither its own field tag nor placeholder bytes.

`ModuleCertBytes` is trusted payload only. Source maps, diagnostics, AI traces,
elaborator traces, tactic traces, and display names do not exist in this schema.
In v0.1, metadata cannot affect trusted hashes because metadata cannot be
encoded into the trusted payload. Future debug / audit sidecars must also remain
outside `export_hash`, `axiom_report_hash`, and `certificate_hash`.

## 11.5 Level Schema

`LevelTable` is topological. Nodes with child levels may reference only smaller
`LevelId` values. Structurally identical level nodes are forbidden duplicates.

```text
LevelId =
  uvar index into level_table

LevelNode =
  0x00 Zero
  0x01 Succ(inner: LevelId)
  0x02 Max(lhs: LevelId, rhs: LevelId)
  0x03 IMax(lhs: LevelId, rhs: LevelId)
  0x04 Param(name: NameId)
```

Only levels after Phase 1 `normalize_level` may be stored. Non-normalized levels
are `NonCanonicalEncoding`.

Canonical `LevelTable` contains reachable normalized level nodes sorted by:

```text
LevelSortKey =
  (height, LevelNodeKey)

height(Zero) = 0
height(Param) = 0
height(Succ x) = height(x) + 1
height(Max x y) = max(height(x), height(y)) + 1
height(IMax x y) = max(height(x), height(y)) + 1
```

`LevelNodeKey` is the canonical bytes of tag and fields. Since `height` is
compared first, child nodes always precede parents. Duplicate `LevelNodeKey`
values are rejected.

## 11.6 Term Schema

`TermTable` is also topological. Nodes with child terms may reference only
smaller `TermId` values. Structurally identical term nodes are forbidden
duplicates.

```text
TermId =
  uvar index into term_table

TermNode =
  0x00 Sort(level: LevelId)
  0x01 BVar(index: uvar)
  0x02 Const(global_ref: GlobalRef, levels: vec<LevelId>)
  0x03 App(fun: TermId, arg: TermId)
  0x04 Lam(type: TermId, body: TermId)
  0x05 Pi(type: TermId, body: TermId)
  0x06 Let(type: TermId, value: TermId, body: TermId)

GlobalRef =
  0x00 Imported(import_index: uvar, name: NameId, decl_interface_hash: hash)
  0x01 Local(decl_index: uvar)
  0x02 LocalGenerated(decl_index: uvar, name: NameId)
  0x03 Builtin(name: NameId, decl_interface_hash: hash)
```

`Imported.import_index` indexes `imports`. `Imported.name` and
`decl_interface_hash` must match an entry in that import's `export_block`.
`Local.decl_index` indexes the current module's declarations.
`LocalGenerated.decl_index` points to an `InductiveDecl` in the current module,
and `LocalGenerated.name` must be a constructor or recursor generated by that
inductive. The checker confirms the matching `ConstructorSpec` / `RecursorSpec`.
`Builtin.name` is a stable name from the checker builtin profile.
`decl_interface_hash` must be deterministically recomputable from the builtin
interface tag. In v0.1 only `Nat`, `Nat.zero`, `Nat.succ`, `Nat.rec`, `Eq`,
`Eq.refl`, and `Eq.rec` are allowed. `Eq.rec` is a builtin axiom interface, so
any declaration referencing it includes `Builtin(Eq.rec)` in its axiom report.

Phase 2 does not support mutual declarations. Local dependencies may reference
only declaration indices smaller than the current declaration. Inductive self
references and generated artifact references inside the same `InductiveDecl`
bundle are not declaration graph cycles.

`Lam` / `Pi` / `Let` store no binder names. Out-of-range de Bruijn indices are
`InvalidBVar`.

Canonical `TermTable` contains reachable term nodes sorted by:

```text
TermSortKey =
  (height, TermNodeKey)

height(Sort _) = 0
height(BVar _) = 0
height(Const _ _) = 0
height(App f a) = max(height(f), height(a)) + 1
height(Lam t b) = max(height(t), height(b)) + 1
height(Pi t b) = max(height(t), height(b)) + 1
height(Let t v b) = max(height(t), height(v), height(b)) + 1
```

`TermNodeKey` is tag and field canonical bytes. Child term fields use child
`term_hash`; level fields use `level_hash`. Duplicate `TermNodeKey` values are
rejected. AST traversal order and hash-map iteration order must not affect table
order.

## 11.7 Declaration Schema

Phase 2 has four logical declaration kinds. Constructors and recursors are not
stored as independent source declarations; the verifier generates kernel
environment declarations from `InductiveDecl`.

```text
DeclCert =
  decl: DeclPayload
  dependencies: vec<DependencyEntry>
  axiom_dependencies: vec<AxiomRef>
  hashes: DeclHashes

DeclPayload =
  0x00 AxiomDecl
  0x01 DefDecl
  0x02 TheoremDecl
  0x03 InductiveDecl

AxiomDecl =
  name: NameId
  universe_params: vec<NameId>
  type: TermId

DefDecl =
  name: NameId
  universe_params: vec<NameId>
  type: TermId
  value: TermId
  reducibility: Reducibility

TheoremDecl =
  name: NameId
  universe_params: vec<NameId>
  type: TermId
  proof: TermId
  opacity: Opacity

InductiveDecl =
  name: NameId
  universe_params: vec<NameId>
  params: vec<BinderType>
  indices: vec<BinderType>
  sort: LevelId
  constructors: vec<ConstructorSpec>
  recursor: option<RecursorSpec>

BinderType =
  type: TermId

ConstructorSpec =
  name: NameId
  type: TermId

RecursorSpec =
  name: NameId
  universe_params: vec<NameId>
  type: TermId
  rules: RecursorRules

RecursorRules =
  minor_start: uvar
  major_index: uvar

Reducibility =
  0x00 Reducible
  0x01 Opaque

Opacity =
  0x00 Opaque

DeclHashes =
  decl_interface_hash: hash
  decl_certificate_hash: hash
```

Phase 2 theorems are always opaque. If a transparent proof should participate in
downstream conversion, represent it as `DefDecl`.

When `InductiveDecl.recursor` is `some`, the verifier regenerates recursor type
and rules from the inductive declaration and compares them with the
`RecursorSpec` in the certificate. Mismatch is
`InductiveGeneratedArtifactMismatch`.

## 11.8 Dependency / Axiom Schema

Dependencies are the duplicate-free set of `Const` references appearing in a
declaration's type, value, proof, constructor type, or recursor type.

```text
DependencyEntry =
  global_ref: GlobalRef
  decl_interface_hash: hash

AxiomRef =
  global_ref: GlobalRef
  name: NameId
  decl_interface_hash: hash
```

`DependencyEntry` and `AxiomRef` are stored in ascending canonical `GlobalRef`
byte order. Explanatory JSON may display only axiom names, but trusted payloads
include `GlobalRef` and `decl_interface_hash`.

Dependency cycles are `DependencyCycle`. Phase 2 does not accept mutual
declarations. Inductive constructors / recursors are internal generated items of
the same `InductiveDecl` bundle, not declaration graph cycles.

## 11.9 Export Block Schema

`ExportBlock` is the public interface downstream modules use to build their
environment. Entries are stored in canonical `name` order.

```text
ExportBlock =
  vec<ExportEntry>

ExportEntry =
  name: NameId
  kind: ExportKind
  universe_params: vec<NameId>
  type: TermId
  body: option<TermId>
  type_hash: hash
  body_hash: option<hash>
  reducibility: option<Reducibility>
  opacity: option<Opacity>
  decl_interface_hash: hash
  axiom_dependencies: vec<AxiomRef>

ExportKind =
  0x00 Axiom
  0x01 Def
  0x02 Theorem
  0x03 Inductive
  0x04 Constructor
  0x05 Recursor
```

`type` is the canonical type term needed by downstream environments. `body` is
used only for transparent / reducible def values. For opaque theorems, axioms,
inductives, constructors, and recursors, `body = none`. `type_hash` must equal
`term_hash(type)`. `body_hash` must equal `term_hash(body)` when `body = some`
and must be `none` when `body = none`.

`body_hash` is only the value hash for transparent / reducible defs. Opaque
theorem proof hashes are not in `ExportBlock`. Inductive `type` is the full
telescope `Pi params indices, Sort sort`. Constructors and recursors generated
from `InductiveDecl` are included as export interfaces.

An import verifier rebuilds a kernel environment from `type` and `body` in the
checked import certificate's `ExportBlock`. Do not build an environment from
hashes alone. If a term inside an imported `ExportBlock` contains
`GlobalRef::Local`, it refers to the declaration index in the imported module,
not the caller module. A caller certificate refers to imported declarations with
`GlobalRef::Imported(import_index, name, decl_interface_hash)`.

## 11.10 Axiom Report Schema

The canonical axiom report is trusted payload, not an audit log.

```text
AxiomReport =
  per_declaration: vec<DeclAxiomReport>
  module_axioms: vec<AxiomRef>

DeclAxiomReport =
  decl_index: uvar
  direct_axioms: vec<AxiomRef>
  transitive_axioms: vec<AxiomRef>
```

`per_declaration` is in declaration order. Each axiom list and `module_axioms`
are in canonical `AxiomRef` order. `safe_for_high_trust`, `contains_sorry`,
`custom_axioms`, and `standard_axioms` are not trusted payload. Recompute them
in the audit view after decode if needed.

## 11.11 Hash Payload Table

Hashes are computed over raw canonical bytes prefixed with an ASCII domain
separator. The domain separator has no length prefix.

```text
level_hash(level) =
  sha256("NPA-LEVEL-0.1" || LevelHashPayload(level))

term_hash(term) =
  sha256("NPA-TERM-0.1" || TermHashPayload(term))

decl_interface_hash(decl) =
  sha256("NPA-DECL-IFACE-0.1" || DeclInterfacePayload(decl))

decl_certificate_hash(decl_cert) =
  sha256("NPA-DECL-CERT-0.1" || DeclCertificatePayload(decl_cert))

axiom_report_hash(report) =
  sha256("NPA-AXIOM-REPORT-0.1" || AxiomReportBytes(report))

export_hash(export_block) =
  sha256("NPA-MODULE-EXPORT-0.1" || ExportBlockBytes(export_block))

certificate_hash(cert) =
  sha256("NPA-MODULE-CERT-0.1" || ModuleCertBytesWithoutCertificateHash(cert))
```

`TermHashPayload` is structural, not table-index based. Child terms use child
`term_hash`, levels use `level_hash`, and global refs use canonical
`GlobalRef` bytes.

```text
TermHashPayload =
  Sort: 0x00 level_hash
  BVar: 0x01 uvar index
  Const: 0x02 GlobalRefBytes vec<level_hash>
  App: 0x03 term_hash(fun) term_hash(arg)
  Lam: 0x04 term_hash(type) term_hash(body)
  Pi: 0x05 term_hash(type) term_hash(body)
  Let: 0x06 term_hash(type) term_hash(value) term_hash(body)
```

`DeclInterfacePayload` includes:

```text
Axiom:
  kind, name, universe_params, type_hash, public_dependency_entries

Def:
  kind, name, universe_params, type_hash, reducibility,
  public_dependency_entries, axiom_dependencies
  value_hash only when reducibility = reducible

Theorem:
  kind, name, universe_params, type_hash, opacity,
  public_dependency_entries, axiom_dependencies

Inductive:
  kind, name, universe_params, params, indices, sort,
  constructors, generated recursor signature hash, generated computation rule hash,
  public_dependency_entries, axiom_dependencies
```

`public_dependency_entries` is derived directly from terms in the public
interface. For axioms and theorems this is the type; for reducible defs, type
and body; for opaque defs, type; for inductives, params / indices / constructor
types / recursor type. Non-axiom dependencies from proofs and opaque bodies
appear only on the certificate-hash side.

`DeclCertificatePayload` includes:

```text
Axiom:
  decl_interface_hash, axiom_dependencies

Def:
  decl_interface_hash, value_hash, dependency entries, axiom_dependencies

Theorem:
  decl_interface_hash, proof_hash, dependency entries

Inductive:
  decl_interface_hash, dependency entries, axiom_dependencies
```

Generated recursor hashes fix generated inductive artifacts:

```text
generated_recursor_signature_hash =
  sha256("NPA-GEN-REC-SIG-0.1" || GeneratedRecursorSignaturePayload)

GeneratedRecursorSignaturePayload =
  None:
    0x00
  Some:
    0x01 recursor_name recursor_universe_params recursor_type_hash

generated_computation_rule_hash =
  sha256("NPA-GEN-COMP-RULE-0.1" || GeneratedComputationRulePayload)

GeneratedComputationRulePayload =
  None:
    0x00
  Some:
    0x01 minor_start major_index
```

When no recursor exists, hash an absence marker rather than changing the shape
of `DeclInterfacePayload`. `recursor_type_hash` is the term hash of the recursor
type. `minor_start` / `major_index` use the same canonical uvar encoding as the
`RecursorRulesSpec` recomputed by the verifier.

## 11.12 Import Store / High-Trust Semantics

Imports are resolved from checked modules, not filesystem paths.

```text
ImportKey =
  module: Name
  export_hash: hash
  certificate_hash: option<hash>

VerifiedModule =
  module: Name
  name_table: vec<Name>
  level_table: vec<LevelNode>
  term_table: vec<TermNode>
  declarations: vec<DeclCert>
  export_hash: hash
  certificate_hash: hash
  export_block: ExportBlock
  axiom_report: AxiomReport
```

`VerifiedModule` is constructed by the verifier from checked canonical payload.
Do not carry a Rust `Decl` vector around as trusted import state. Import-side
kernel environments are decoded from canonical tables / declarations in
`VerifiedModule`. If a future kernel API can consume interface fragments
directly, this can shrink to reconstruction from `ExportBlock` alone.

In normal mode, it is enough for `VerifierSession` to contain a `VerifiedModule`
matching `ImportEntry.module` and `ImportEntry.export_hash`.
`ImportEntry.certificate_hash` may be absent; if present, it must match.

High-trust mode requires all of:

```text
- ImportEntry.certificate_hash is some
- module / export_hash / certificate_hash match VerifiedModule
- that VerifiedModule was checked by verify_module_cert in the current VerifierSession
- policy does not allow forbidden axioms or synthetic sorry
```

Do not inject externally constructed `VerifiedModule` values as high-trust
imports.

## 11.13 Structured Error Enum

Phase 2 failures return structured errors, not only strings.

```rust
pub enum HashObject {
    Level,
    Term,
    DeclInterface,
    DeclCertificate,
    ExportBlock,
    AxiomReport,
    ModuleCertificate,
}

pub enum CertError {
    DecodeError,
    UnsupportedFormat { format: String, core_spec: String },
    UnsupportedEncoding { tag: u8 },
    NonCanonicalEncoding { object: &'static str },
    HashMismatch { object: HashObject, expected: Hash, actual: Hash },
    ImportHashMismatch { module: ModuleName },
    MissingImportCertificateHash { module: ModuleName },
    ImportCertificateHashMismatch { module: ModuleName },
    ImportNotVerifiedInSession { module: ModuleName },
    DuplicateImportEnvKey { module: ModuleName, export_hash: Hash },
    DuplicateName { name: ModuleName },
    UnknownDependency { name: ModuleName },
    DependencyCycle { name: ModuleName },
    AxiomReportMismatch { decl: Option<ModuleName> },
    ForbiddenAxiom { axiom: ModuleName },
    SorryDenied { axiom: ModuleName },
    UnresolvedMetavariable,
    InvalidBVar { index: u32 },
    InductiveGeneratedArtifactMismatch { name: ModuleName },
    Kernel(npa_kernel::Error),
}
```

CLI and diagnostics produce human-readable messages from this enum. Tests check
the enum variant and key fields directly.

---

# 12. Phase 2 Test Cases

Phase 2 completion is checked by these tests.

## 12.1 Golden Certificate

Generate `.npcert` from at least these core declarations and fix the bytes or
hashes as golden fixtures.

```text
- id
- const
- Nat
- Eq
- Nat.add
- add_zero
```

Expected:

```text
- the same input produces the same .npcert bytes
- the Phase 2 certificate verifier passes without source
- export_hash / certificate_hash / axiom_report_hash match recomputation
```

## 12.2 Canonicalization / Hash Stability

```text
- terms that differ only by binder names have the same term_hash
- canonical declaration order is stable when input order expresses the same dependencies
- reordering import inputs still yields stable canonical import order and module hash
- name / level / term table generation order is independent of internal traversal order
```

## 12.3 Hash-Role Differential Tests

```text
- changing the body of a transparent def changes decl_interface_hash and export_hash
- changing only the body of an opaque def preserves export_hash when type, reducibility, and axiom dependencies are unchanged
- in that case, decl_certificate_hash and certificate_hash still change
- changing only the proof of an opaque theorem preserves export_hash when type, opacity, and axiom dependencies are unchanged
- in that case, decl_certificate_hash and certificate_hash still change
- changing the proof of an opaque theorem changes export_hash when axiom dependencies change
- axiom_report_hash changes only when the canonical axiom report changes
```

## 12.4 Mutation / Rejection

Create invalid certificates and confirm they are rejected with structured
errors.

```text
- change one byte of a proof body
- tamper with term_hash / declaration hash / export_hash / certificate_hash / axiom_report_hash
- remove an actually used axiom from the axiom report
- confirm unresolved metavariables cannot be represented in the trusted schema
- insert an unknown term tag
- use nonminimal ULEB128
- add an unused term table entry
- break table order / import order / declaration order
- insert a map-like noncanonical representation into a hash payload
```

Expected errors include:

```text
HashMismatch
AxiomReportMismatch
UnresolvedMetavariable
UnsupportedEncoding
NonCanonicalEncoding
Kernel(npa_kernel::Error)
```

depending on the cause.

## 12.5 Import / High-Trust Mode

```text
- normal mode requires import export_hash match
- normal mode allows missing import certificate_hash
- high-trust mode rejects missing certificate_hash
- high-trust mode rejects certificate_hash mismatch
- high-trust mode rejects import certificates not checked by the same verifier session
```

## 12.6 Axiom Policy / Source Independence

```text
- policy can reject forbidden axioms
- deny_sorry policy can reject synthetic sorry axioms
- source maps / diagnostics / AI traces do not exist in trusted schema and cannot be encoded into hash input
- verification succeeds from only .npcert and import store after deleting source files
```

## 12.7 Producer Separation

Check Human / AI producer separation with tests:

```text
- Human-derived and AI-derived CoreModule values representing the same core declarations produce identical .npcert bytes and hashes
- changing producer_profile / producer_run_id / model name / score / diagnostics in sidecars does not change .npcert bytes or hashes
- Accepted from check_core_decl_candidates cannot be treated directly as VerifiedModule
- .npcert built from an Accepted candidate does not enter the trusted import store until verify_module_cert passes
- invalid prior token is a batch-level Err(CertError), not a per-candidate rejection
- noncanonical CandidateBatch.imports is a batch-level Err(CertError::NonCanonicalEncoding)
- duplicate ProducerImportEnvKey(module, export_hash) in a batch is Err(CertError::DuplicateImportEnvKey)
- CandidateBatchResult.statuses has the same length and order as input candidates
- build_module_cert_from_checked_candidates rejects token chain / pre_env_fingerprint / post_env_fingerprint mismatch
- build_module_cert_from_checked_candidates rejects producer_limits_hash(token.limits) mismatch
- producer public env / prior chain fingerprints are deterministically recomputable from canonical bytes and domain separators
- canonical_import_env_keys(imports) and canonical_import_export_views(imports) preserve the same order, so GlobalRef::Imported(import_index, ...) refers to the same import
- changing only an imported module proof body while preserving module name and export_hash preserves producer public env fingerprint
- producer public env fingerprint axiom dependencies are recomputed with the same compute_axiom_deps as certificate generation
- changing only an opaque theorem proof / opaque def body preserves producer public env fingerprint when public interface and axiom dependencies are unchanged, while prior chain fingerprint changes because decl_certificate_hash changes
- ProducerLimits canonical hash and stricter_or_equal are deterministic
- wrong AI preview hashes are ignored by token validation / build_module_cert / verify_module_cert, which use recomputation only
- AI-derived candidates with unresolved metavariables / placeholders / pretty-only GlobalRef are rejected
- failure of one candidate in a batch does not make other candidate results depend on failure order or cache state
- enabling or disabling cache produces the same .npcert bytes from the same accepted module
```

---

# 13. Phase 2 Completion Criteria

Phase 2 is complete when:

```text
- core terms can be converted to canonical binary
- canonical binary satisfies core-spec v0.1 byte-level conditions and rejects noncanonical encoding
- changing binder names does not change term hash
- imports carry required export_hash and high-trust-required certificate_hash
- import entries are limited to module name / export_hash / optional certificate_hash, and declaration lists are derived from import export_block
- def/theorem/axiom/inductive declarations carry declaration hashes
- changing a transparent def body changes the interface hash
- changing only an opaque def body preserves export_hash when type, reducibility, and axiom dependencies are unchanged
- changing an opaque def body changes decl_certificate_hash and module certificate_hash
- changing an opaque theorem proof changes decl_certificate_hash and module certificate_hash
- changing only an opaque theorem proof preserves export_hash when type, opacity, and axiom dependencies are unchanged
- changing an opaque theorem proof changes export_hash when axiom dependencies change
- axiom dependencies can be computed per declaration
- module-wide axiom reports can be emitted in canonical order
- audit views such as safe_for_high_trust are recomputed, not trusted as stored values
- the checker can recheck using only .npcert and .npcert files in the import store, and the kernel checks only decoded declarations
- the Phase 2 checker is the certificate verifier using the same Rust kernel, with responsibilities separated from the Phase 8 independent checker
- verification completes without source code, source maps, or AI traces
- Human producer / AI producer are separated as untrusted layers up to CoreModule or CoreDeclCandidate
- AI candidate fast-path success is distinct in type/API and operation from .npcert verification success
- tests confirm producer metadata / sidecars do not enter trusted payloads or hashes
- the API / byte schema / hash payload / error enum from chapter 11 is implemented
- the golden / stability / mutation / high-trust / source-independence / producer separation tests from chapter 12 pass automatically
```

## 13.1 Current Implementation Status

`crates/npa-cert` already implements the Phase 2 trusted certificate verifier
for:

```text
- generating ModuleCert from CoreModule
- canonical binary encode/decode
- checking canonical bytes by matching reencoded bytes after decode
- checking canonical order and reachability of name / level / term tables
- checking canonical order of imports / declarations / export block / axiom report
- recomputing level / term / declaration / export / axiom report / module certificate hashes
- checking normal / high-trust import policy
- reconstructing kernel environment only from the verified import store
- checking axiom reports and axiom policy from recomputed values, not stored booleans
- checking inductive constructor / recursor export and generated artifact mismatch
- passing decoded declarations to the Phase 1 Rust kernel for rechecking
```

Items intentionally excluded from the v0.1 trusted payload:

```text
- source map
- diagnostics
- display name
- elaborator trace
- tactic trace
- AI trace
- unresolved metavariable / hole / placeholder
```

These are not trusted hash inputs. If metadata is needed, keep it as a debug
sidecar outside `.npcert`.

The Phase 8 independent checker is later work that rechecks this `.npcert`
schema in another implementation or process. It is not part of Phase 2.

The Human producer / AI producer separation, `CoreDeclCandidate` /
`CheckedDeclCandidate` fast path, producer sidecar, and producer separation tests
defined in 7.1, 11.1.1, and 12.7 are currently detailed design. They are not
part of the already implemented trusted verifier items in `crates/npa-cert`.
To count them as implemented, add opaque `CheckedDeclCandidate` tokens,
producer public env fingerprint / prior chain validation,
`check_core_decl_candidates`, `build_module_cert_from_checked_candidates`,
`ProducerLimits` canonical hash / strictness checks, canonical bytes for
producer public env / prior chain fingerprints, and the producer separation
tests in 12.7.

---

# 14. Key Design Points

The most important part of Phase 2 is separating hash roles.

```text
term_hash:
  hash of the core term itself

decl_interface_hash:
  hash of the declaration meaning visible downstream

decl_certificate_hash:
  hash of the whole declaration including proof/value body

export_hash:
  hash of the module public interface

certificate_hash:
  hash of the trusted payload excluding certificate_hash itself

axiom_report_hash:
  hash of the canonical axiom report
```

The axiom report is not a mere log. It is verification data as important as the
hashes.

One-sentence summary:

```text
Fix canonical core AST as a binary certificate,
assign hashes to imports, declarations, and modules,
make axiom dependencies recomputable per declaration,
and provide a source-free format the kernel can recheck.
```
