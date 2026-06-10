# Phase 3 AI Profile: Machine Surface

This document is the design and implementation plan for **AI-facing Phase 3**
of NPA.

The existing `develop/phase3-human.md` designs a surface language that is easy
for humans to read and write, then lowers it to canonical core AST. In AI proof
search, by contrast, many candidates are generated and checked quickly under
the assumption that most will fail. Therefore AI-facing Phase 3 removes
human-facing conveniences and uses an explicit input format close to canonical
core.

This AI-facing input format is called **Machine Surface**.

Machine Surface is not a trusted language. The parser / resolver / elaborator /
AI output are all untrusted layers. Final correctness is confirmed when the
Phase 1 kernel and Phase 2 certificate verifier check fully explicit core
declarations / proof terms.

Implementation status (2026-05-21):

```text
- Machine Surface M0-M9 are implemented as the AI fast path in crates/npa-frontend / crates/npa-api / crates/npa-tactic
- Human Surface is a separate profile in develop/phase3-human.md and is not placed in the default paths for AI candidate generation, tactic execution, search, replay, or verify
- Non-regression conditions are fixed by Machine Surface accepted/rejected syntax, term-source canonical hash, Phase 7/9 fixtures, and scripts/phase9-regression.sh
```

---

# 1. Purpose

The purpose of Machine Surface is to check AI-generated proof candidates
quickly, deterministically, and in a repair-friendly way.

Ordinary human-facing Phase 3:

```text
human source
  ↓ parser with active notation table
surface AST with notation nodes
  ↓ name resolution + notation candidate collection
resolved AST with overloaded names / notation candidates
  ↓ elaboration + metavariable solving
fully explicit core AST
  ↓ certificate generation
canonical certificate
```

AI-facing Phase 3:

```text
machine source / structured AI request
  ↓ fixed parser
machine AST
  ↓ direct resolver
resolved machine AST
  ↓ explicit elaboration
fully explicit core AST
  ↓ kernel check
  ↓ certificate generation
canonical certificate
```

For AI-facing use, prioritize the following.

```text
- do not depend on an active notation table
- do not depend on open / namespace
- do not create overload candidate transactions
- do not automatically insert implicit term arguments
- do not represent unresolved holes
- return failures as structured errors
- return the same core hash / error from the same input
```

---

# 2. Trust Boundary

Machine Surface input is not trusted.

```text
Untrusted:
  AI output
  Machine Surface parser
  direct resolver
  elaborator
  repair suggestion
  structured hint
  source span

Trusted:
  deterministic result from the Phase 1 Rust kernel checking fully explicit core
  canonical certificate bytes / hash and Phase 2 certificate verifier result
  Phase 8 independent checker result
```

Machine-Surface-specific information is not kept in certificates.

```text
Not included in trusted payload:
  machine source text
  AI prompt / completion / trace
  repair suggestion
  source span
  display name
  notation metadata
  implicit argument metadata
  unresolved metavariable / hole / placeholder
```

Even if AI outputs a fully explicit term, that is not a proof. Only terms that
pass kernel check and certificate check are accepted as proofs.

---

# 3. Design Policy

Machine Surface is a subset of human-facing Phase 3, but implementation-wise it
is built as AI-first from the beginning. Instead of adding human-facing features
and disabling them with flags, first build a parser / resolver / elaborator with
no ambiguity.

## 3.1 Features Not Used

The Machine Surface MVP does not use the following.

```text
- notation declaration
- prefix / postfix / infix notation
- overloaded notation
- overloaded short name
- open
- namespace
- implicit term argument insertion
- unannotated lambda binder
- unannotated declaration binder
- unannotated let
- unresolved hole
- named hole reuse
- source-level axiom declaration
- source-level inductive declaration
- source-level recursive definition syntax
- typeclass search
- coercion search
- numeric literal overload
```

These may be added in the future as part of the human-facing surface language or
another profile, but they are not included in the AI-first Phase 3 MVP.

## 3.2 Features Used

The Machine Surface MVP uses the following.

```text
module item:
  import
  def
  theorem

term:
  fully qualified global name
  local name
  explicit universe application
  Sort / Type / Prop
  application
  typed lambda
  typed forall / Pi
  typed let
  annotation
  parenthesized term
```

AI outputs a form like the following, not `n + 0 = n`.

```npa
Eq.{1} Nat (Nat.add n Nat.zero) n
```

For theorems and definitions with implicit arguments, term arguments are made
explicit with `@`.

```npa
@Eq.refl.{1} Nat n
```

---

# 4. Machine Surface Syntax

## 4.1 Module grammar

```text
machine_module ::=
    import_item* machine_item*

import_item ::=
    "import" qual_name

machine_item ::=
    machine_def
  | machine_theorem

machine_def ::=
    "def" qual_name universe_params? machine_binder* ":" machine_term ":=" machine_term

machine_theorem ::=
    "theorem" qual_name universe_params? machine_binder* ":" machine_term ":=" machine_term

machine_binder ::=
    "(" ident ":" machine_term ")"
```

`import` appears only at the beginning of a module.
The name of `def` / `theorem` is treated directly as the declaration name, not
as a name relative to the current namespace. Machine Surface MVP has no
`namespace`, so it has no declaration name conversion rule.

## 4.2 Term grammar

```text
machine_term ::=
    machine_app
  | "fun" machine_binder+ "=>" machine_term
  | "forall" machine_binder+ "," machine_term
  | "let" ident ":" machine_term ":=" machine_term "in" machine_term
  | machine_app ":" machine_term

machine_app ::=
    machine_atom machine_atom*

machine_atom ::=
    term_name universe_args?
  | "@" term_name universe_args?
  | "Prop"
  | "Type" level?
  | "Sort" level
  | "(" machine_term ")"

term_name ::= name_component ("." name_component)*

universe_params ::= ".{" ident ("," ident)* "}"
universe_args   ::= ".{" level ("," level)* "}"

level ::=
    natural
  | ident
  | "succ" level
  | "max" level level
  | "imax" level level
```

`name_component` is the spelling of a name component in machine source, and uses
the same ASCII subset as identifiers. Components with the same spelling as
keywords are keywords in grammar positions read as keywords, but are name
components in positions read as components of dotted names. For example, `succ`
in `Nat.succ` is a name component, not the `succ` keyword in the level grammar.

The lexer / parser must not accept reserved tokens as ordinary binder `ident`,
universe parameter `ident`, or unqualified / head `term_name`. The following
tokens are treated only as the corresponding term syntax or dedicated keywords.

```text
reserved tokens:
  import
  def
  theorem
  forall
  fun
  let
  in
  Prop
  Type
  Sort
  succ
  max
  imax
  open
  namespace
  match
  with
```

However, in components after `.` in `term_name`, the spellings above can also be
used as name components. For example, `Nat.succ`, `M.Type`, and `M.match` can be
parsed as dotted `term_name`. A standalone `Prop` is always a Prop atom, `Type`
is a Type atom, and `Sort` is a Sort atom; they must not be canonicalized as
`NameAtom("Prop")` / `NameAtom("Type")` / `NameAtom("Sort")`. `succ` / `max` /
`imax` are keywords inside the level grammar and cannot be used as level
parameter names.

For the Phase 7 forbidden-token filter, the Machine Surface lexer exposes a
token-only API that does not go through the parser / resolver / elaborator. This
is a lexical API separate from `MachineSurfaceMode::Complete` / `Repair`, and is
not a mode that accepts source. The token-only API uses the same lexical rules
and longest-match rules as the Machine Surface parser, and returns a
pre-parser `MachineDiagnostic` on lexical errors.

```rust
pub struct MachineSurfaceToken {
    pub kind: MachineSurfaceTokenKind,
    pub spelling: String,
    pub span: Span,
}

pub enum MachineSurfaceTokenKind {
    IdentLike,
    Reserved,
    Dot,
    Punctuation,
    Natural,
    StringLiteral,
    Whitespace,
    Comment,
    ExternalCommand,
}

pub fn lex_machine_surface_tokens(
    source: &str,
) -> Result<Vec<MachineSurfaceToken>, MachineDiagnostic>;
```

`spelling` is the token substring exactly as it appears in the source after JSON
decode. Do not perform case folding, Unicode normalization, name resolution, or
keyword aliasing. In the token-only API, `term_name` is returned as per-component
`IdentLike` / `Reserved` plus `Dot`. Therefore `Std.UnsafeName.foo` only has the
`UnsafeName` component and does not become an `unsafe` token. By contrast,
`Std.unsafe.foo` contains a component with spelling `unsafe`. Callers of the
token-only API must not re-lex the contents of `StringLiteral`. The current MVP
Machine Surface grammar has no external command / shell escape, so
`ExternalCommand` is unreachable. Even if that token is introduced in the
future, the token-only API returns it as a dedicated token kind independently of
whether the parser accepts it.

The MVP does not use `->` / `→`. Function types are written as `forall (x : A),
B`. Anonymous binders are also not used.

## 4.3 Syntax rejection

The following are rejected as parse errors or dedicated diagnostics.

```npa
open Nat
namespace Nat
infixl:65 " + " => Nat.add
axiom choice : ...
inductive Nat : Type where ...
fun x => x
let x := Nat.zero in x
_
?m
n + Nat.zero
```

---

# 5. Name Resolution

## 5.1 Global names

Global names use fully qualified exact match only.

Allowed:

```text
Nat
Nat.zero
Nat.add
Eq
Eq.refl
Std.Logic.False.elim
```

Rejected:

```text
zero
add
refl
```

Term name resolution happens only in `elaborate_machine_term_check` / module
elaboration, not in `canonicalize_machine_term_source`. Resolution proceeds in
the following order.

```text
1. If term_name is @ term_name, has universe_args, or has component_count > 1:
   try only global exact match. Do not inspect local context.
2. Otherwise, for a one-component term_name:
   if it exists in local context as an unqualified local name, it is local.
   otherwise try global exact match.
3. All other cases are UnknownName.
```

However, reject cases where a local name shadows the same fully qualified root
name as a global declaration.

```npa
-- reject
theorem bad (Nat : Type) (x : Nat) : Nat := x
```

Reason:

```text
- eliminate suffix lookup
- eliminate open scope
- eliminate overload candidates
- limit AI repair to fully qualified name correction
```

## 5.2 Imported names

Imports are resolved from Phase 2 verified modules.

```text
source:
  import Std.Nat.Basic

compile request:
  verified_imports:
    Std.Nat.Basic + export_hash + optional certificate_hash
```

The Machine Surface resolver does not search the filesystem or network for
imports. It sees only the verified import set passed by the compile request.

Imported globals are fixed by `name + decl_interface_hash` in the export block.
Even in the term-only API, the `decl_interface_hash` in
`MachineGlobalScopeEntry` must match the checked environment view held by
`MachineKernelEnvView`; mismatches are rejected before elaboration.

Future AI APIs should allow passing structured refs directly instead of names in
source text.

```json
{
  "kind": "imported",
  "module": "Std.Nat.Basic",
  "name": "Nat.add",
  "decl_interface_hash": "sha256:..."
}
```

Structured refs are also untrusted. The Phase 2 certificate verifier rechecks
them against import entries and export blocks.

---

# 6. Elaboration

Machine Surface elaboration assumes explicitness.

```text
Enabled:
  explicit application
  typed lambda
  typed Pi
  typed let
  annotation
  explicit universe args

Disabled:
  implicit term arg insertion
  notation candidate selection
  overload resolution
  hole goal creation
  typeclass search
  coercion search
```

## 6.1 Application

`f a` is elaborated with the following steps.

```text
1. infer f
2. reduce type(f) to WHNF
3. confirm the head is Pi
4. check a against the domain
5. instantiate the codomain
```

Even if caller-supplied metadata for a function type has implicit binders,
Machine Surface does not automatically insert implicit term arguments. To pass
an explicit argument to an implicit binder position, use a head with `@`.

```npa
@Eq.refl.{1} Nat n
```

`Eq.refl n` is rejected as `ImplicitArgumentRequired` only when the callable
profile supplied by the caller fixes the first term binder of `Eq.refl` as
implicit. In protocols such as Phase 5 AI MVP v1 where the caller passes an
all-explicit callable profile, that profile is authoritative for this judgment.
In that case `@` is allowed as an exact-match / canonical source marker, but no
implicit binder consumption occurs, and `ImplicitArgumentRequired` is not raised
for a missing implicit term binder. The Machine Surface parser / elaborator must
not infer implicit profiles from a server-local registry, original source, or
pretty metadata. In the current implementation where `MachineTermElabContext`
has no callable profile field, the Phase 5 adapter passes equivalent metadata
through a wrapper context or explicit argument. Machine Surface unit tests for
implicit binder behavior also specify in the fixture which callable profile is
used.

## 6.2 Lambda

Lambda binders always have type annotations.

```npa
fun (x : Nat) => x
```

The following is rejected.

```npa
fun x => x
```

The MVP does not infer binder types from expected types.

## 6.3 Pi / forall

`forall` binders also always have type annotations.

```npa
forall (x : Nat), Nat
```

`A -> B` is not included as syntax in the MVP.

## 6.4 Let

`let` also always has a type annotation.

```npa
let x : Nat := Nat.zero in x
```

The following is rejected.

```npa
let x := Nat.zero in x
```

## 6.5 Universe

Declaration-level universe parameters are explicit.

```npa
def id.{u} (A : Sort u) (x : A) : A := x
```

When using polymorphic constants, AI-generated source should also make universe
args explicit.

```npa
@Eq.refl.{1} Nat n
```

In the MVP implementation, omitted universe arguments for imported polymorphic
constants may become internal universe metavariables, but they are rejected if
unsolved at the end of complete mode. The standard AI-generated form uses
explicit universes.

---

# 7. Modes

Machine Surface has two modes.

```rust
pub enum MachineSurfaceMode {
    Complete,
    Repair,
}
```

## 7.1 Complete mode

Complete mode checks source used as a certificate candidate.

Rejected:

```text
- unresolved UserHole
- unresolved SyntheticImplicit
- unresolved UniverseMeta
- OverloadedRef / OverloadedApp
- notation
- open / namespace
- source-level axiom
- source-level inductive
- unknown global name
- short global name
```

On success, it returns a fully explicit core module / term.

```json
{
  "status": "complete",
  "core_hash": "sha256:...",
  "contextual_core_hash": "sha256:...",
  "constants": [
    { "name": "Eq.refl", "decl_interface_hash": "sha256:..." },
    { "name": "Nat", "decl_interface_hash": "sha256:..." }
  ],
  "ready_for_certificate": true
}
```

## 7.2 Repair mode

Repair mode returns structured errors for the AI repair loop.

Example:

```json
{
  "status": "error",
  "error": {
    "kind": "implicit_argument_required",
    "function": "Eq.refl",
    "binder_index": 0
  },
  "suggestions": [
    {
      "replacement": "@Eq.refl.{1} Nat n"
    }
  ]
}
```

Repair mode suggestions are not trusted. A suggestion is accepted only after it
is resubmitted and passes Complete mode plus kernel / certificate checks.

---

# 8. Structured Errors

Errors in AI-facing Phase 3 are enum-centered, not prose-centered.

```rust
pub enum MachineErrorKind {
    ParseError,
    UnsupportedItem,
    UnsupportedSyntax,
    ImportAfterItem,
    ImportResolutionError,
    MissingVerifiedImport,
    UnknownGlobalName,
    ShortGlobalName,
    AmbiguousGlobalName,
    GlobalShadowedByLocal,
    UnknownLocalName,
    DuplicateDeclaration,
    DuplicateUniverseParam,
    UnknownUniverseParam,
    ImplicitArgumentRequired,
    MissingExplicitUniverse,
    UnannotatedBinder,
    UnannotatedLet,
    HoleNotAllowed,
    ExpectedFunctionType,
    ExpectedSort,
    TypeMismatch,
    TooManyArguments,
    TooFewArguments,
    UnsolvedUniverseMeta,
    KernelRejected,
    CertificateRejected,
}
```

Diagnostic payloads include the following where possible.

```text
- source span
- head symbol
- expected type core hash
- actual type core hash
- target core hash
- constants in term
- candidate fully qualified names
- suggested machine replacement
```

However, source spans and suggestions do not enter trusted payloads. The
implementation separates `MachineDiagnostic.payload` and
`MachineDiagnostic.suggestions`, and treats repair suggestions as auxiliary
information for display-only / retry input.

```rust
pub struct MachineDiagnosticPayload {
    pub head_symbol: Option<String>,
    pub expected_hash: Option<Hash>,
    pub actual_hash: Option<Hash>,
    pub target_hash: Option<Hash>,
    pub expected_universe_args: Option<usize>,
    pub actual_universe_args: Option<usize>,
    pub candidates: Vec<MachineRepairCandidate>,
}

pub enum MachineRepairSuggestionKind {
    InsertExplicitArguments,
    InsertExplicitUniverseArguments,
    UseFullyQualifiedName,
}

pub struct MachineRepairSuggestion {
    pub kind: MachineRepairSuggestionKind,
    pub replacement: Option<String>,
    pub candidates: Vec<MachineRepairCandidate>,
}
```

---

# 9. Public API

The MVP crate is `crates/npa-frontend`.
However, public APIs use names that make clear they are Machine Surface, not a
human-facing surface frontend.

```rust
pub struct MachineModule {
    pub name: ModuleName,
    pub items: Vec<MachineItem>,
}

pub struct MachineCompileOptions {
    pub mode: MachineSurfaceMode,
    pub allow_universe_meta: bool,
}

pub struct MachineTermElabContext {
    // Opaque. Constructed only by Phase 3 builders from verified modules and checked current decls.
    // Callers can read through accessor methods, but cannot mutate global scope coordinates after construction.
}

pub struct MachineKernelEnvView {
    // Opaque checked environment built from verified imports and checked current decls.
    // The view also carries the exact declaration interface hashes allowed for each global name.
    // Raw Env + hash side-table constructors are not part of the public API.
}

pub struct MachineCheckedCurrentDecl {
    pub name: Name,
    pub source_index: u64,
    pub decl_interface_hash: Hash,
    pub decl: npa_kernel::Decl,
}

pub struct MachineCheckedCurrentGeneratedDecl {
    pub name: Name,
    pub parent_source_index: u64,
    pub decl_interface_hash: Hash,
}

impl MachineTermElabContext {
    pub fn from_verified_modules(
        direct_verified_modules: &[VerifiedModule],
        available_verified_modules: &[VerifiedModule],
        local_context: Vec<LocalDecl>,
        universe_params: Vec<String>,
    ) -> Result<MachineTermElabContext, MachineDiagnostic>;

    pub fn from_verified_modules_and_current_decls(
        direct_verified_modules: &[VerifiedModule],
        available_verified_modules: &[VerifiedModule],
        checked_current_decls: &[MachineCheckedCurrentDecl],
        current_generated_decls: &[MachineCheckedCurrentGeneratedDecl],
        local_context: Vec<LocalDecl>,
        universe_params: Vec<String>,
    ) -> Result<MachineTermElabContext, MachineDiagnostic>;
}

pub struct MachineGlobalScope {
    // Opaque exact-name lookup scope. Raw entry-vector constructors are not part of the public API.
}

pub enum MachineGlobalScopeEntry {
    Imported {
        name: Name,
        import_index: u32,
        decl_interface_hash: Hash,
    },
    CurrentModule {
        name: Name,
        source_index: u64,
        decl_interface_hash: Hash,
    },
    CurrentGenerated {
        name: Name,
        parent_source_index: u64,
        decl_interface_hash: Hash,
    },
}

pub struct MachineTermSourceCanonical {
    pub source: String,
    pub canonical_bytes: Vec<u8>,
    pub canonical_hash: Hash,
}

pub fn parse_machine_module(
    file_id: FileId,
    source: &str,
) -> Result<MachineModule, MachineDiagnostic>;

pub fn resolve_machine_module(
    module: MachineModule,
    verified_imports: &[VerifiedImport],
) -> Result<ResolvedMachineModule, MachineDiagnostic>;

pub fn resolve_machine_module_with_options(
    module: MachineModule,
    verified_imports: &[VerifiedImport],
    options: &MachineCompileOptions,
) -> Result<ResolvedMachineModule, MachineDiagnostic>;

pub fn elaborate_machine_module(
    module: ResolvedMachineModule,
    verified_imports: &[VerifiedImport],
    options: &MachineCompileOptions,
) -> Result<CoreModule, MachineDiagnostic>;

pub fn compile_machine_source_to_core(
    file_id: FileId,
    module_name: ModuleName,
    source: &str,
    verified_imports: &[VerifiedImport],
    options: &MachineCompileOptions,
) -> Result<CoreModule, MachineDiagnostic>;

pub fn compile_machine_source_to_certificate(
    file_id: FileId,
    module_name: ModuleName,
    source: &str,
    verified_imports: &[VerifiedModule],
    options: &MachineCompileOptions,
) -> Result<ModuleCert, MachineDiagnostic>;
```

Term-only APIs are also provided for Phase 5 / Phase 7.

```rust
pub fn canonicalize_machine_term_source(
    source: &str,
) -> Result<MachineTermSourceCanonical, MachineDiagnostic>;

pub fn lex_machine_surface_tokens(
    source: &str,
) -> Result<Vec<MachineSurfaceToken>, MachineDiagnostic>;

pub fn elaborate_machine_term_check(
    source: &str,
    context: &MachineTermElabContext,
    expected: &npa_kernel::Expr,
    options: &MachineCompileOptions,
) -> Result<MachineTermCheckResult, MachineDiagnostic>;

pub struct MachineTermCheckResult {
    pub expr: npa_kernel::Expr,
    pub inferred_type: npa_kernel::Expr,
    pub core_hash: Hash,
    pub contextual_core_hash: Hash,
    pub constants: Vec<MachineResolvedConstant>,
}

pub struct MachineResolvedConstant {
    pub name: Name,
    pub decl_interface_hash: Hash,
}

pub struct MachineTermAst {
    // parsed term AST; concrete fields are implementation-local.
}

pub fn decode_machine_term_source_canonical(
    canonical_bytes: &[u8],
) -> Result<MachineTermAst, MachineDiagnostic>;

pub fn elaborate_machine_term_infer_from_ast(
    ast: &MachineTermAst,
    context: &MachineTermElabContext,
    options: &MachineCompileOptions,
) -> Result<(npa_kernel::Expr, npa_kernel::Expr), MachineDiagnostic>;
```

`MachineTermCheckResult.core_hash` is the Phase 1 core expression structural
hash without owner context. Its hash domain is `NPA-PHASE1-EXPR-0.1`, separate
from Phase 2 `NPA-TERM-0.1`.
`MachineTermCheckResult.contextual_core_hash` commits not only to the structure
of the elaborated core term, but also to the referenced global
`MachineGlobalScopeEntry` coordinates and `decl_interface_hash`. Imports /
checked current declarations / current generated declarations with the same
display name but different `decl_interface_hash` values become different
hashes. The hash domain is `NPA-PHASE3-MACHINE-TERM-CONTEXT-0.1`.
`contextual_core_hash` must not be treated as the same value as Phase 2
`term_hash` or Phase 5 `MachineExprView.core_hash`.

`MachineTermAst` is a parsed term AST decoded from
`"npa.phase3.machine-term-source.v1"` canonical bytes. Concrete fields are
opaque data that cannot be constructed from the public API; external callers
obtain them through the decoder. The decoder does not accept post-resolver
internal nodes that the parser cannot generate, such as a `Local` term tag. The
return value of `elaborate_machine_term_infer_from_ast` is
`(elaborated_core_term, inferred_type)`. This API is used by paths such as Phase
9 NaturalLanguageFormalization that receive term-source canonical bytes as
replay input and do not yet have an expected type. Do not restore a source
string from canonical bytes or go through a pretty printer.

The local context in `MachineTermElabContext` is passed in outer-to-inner order.
The last local becomes de Bruijn index `0`. `expected` is a core expression
interpreted under the same local context / universe parameter context.

The global scope inside `MachineTermElabContext` is a closed map for exact name
lookup. The resolver must not read filesystem, network, global package cache,
or `open` / namespace state. Phase 5 builds this scope from public exports of
direct imports, checked current declarations, and current generated
constructors / recursors. `direct_verified_modules` are only the direct imports
that enter this lookup scope. `available_verified_modules` are the closure used
by the kernel env to resolve transitive dependencies needed for type checking
direct imports; their exports must not be added to the lookup scope. The
internal `kernel_env` is a checked environment built from the same verified
imports / checked current declarations, and must not be used to increase global
name lookup candidates. It is used only as input to type retrieval, reduction,
conversion, and kernel check.

`Imported.import_index` is the index in the canonical direct import order chosen
by Phase 5 / Phase 4. `CurrentModule.source_index` and
`CurrentGenerated.parent_source_index` are source_index coordinates inside a
Phase 5 session. As with the Phase 2 `GlobalRef::LocalGenerated` dependency,
`CurrentGenerated.decl_interface_hash` must match the `decl_interface_hash` of
the parent checked current declaration. If the Phase 2 declaration index differs
from source_index during final certificate generation, Phase 5 verify rewrites
`GlobalRef::Local` / `GlobalRef::LocalGenerated` to certificate-local indexes.

`canonicalize_machine_term_source` turns the parsed Machine Surface term AST,
not raw source text, into canonical bytes.
`MachineTermSourceCanonical.canonical_hash` is `hash(canonical_bytes)`, the
Phase 3 term-source hash. This is not a Phase 4 wrapper hash. Phase 4
`MachineTermSource.canonical_hash` is calculated on the Phase 4 side from the
Phase 4 tag and `canonical_bytes`. `canonicalize_machine_term_source` does not
receive `MachineTermElabContext` and does not perform global name resolution,
local context lookup, type checking, reduction, or kernel environment lookup.
Only `elaborate_machine_term_check` performs context-dependent checks. The
`NameAtom` below is a parser-level syntax variant, not a resolution result to
core `GlobalRef` / local de Bruijn. One-component names such as `Nat`, `Eq`,
and `n` are all encoded at this stage as `NameAtom` by the same rules. Local /
global classification by local context, shadowing checks, and UnknownName are
judged only in `elaborate_machine_term_check`. The string / list / option /
unsigned integer / hash primitives in this block use the same minimal unsigned
LEB128 length + UTF-8 bytes / raw digest bytes as Phase 2 canonical encoding.

```text
Machine Surface term-source canonical bytes:
  - tag "npa.phase3.machine-term-source.v1"
  - parsed term AST encoded with the canonical grammar below

Term canonical bytes:
  - NameAtom:
      variant tag
      name component list:
        component count as minimal unsigned LEB128 u32
        each component as UTF-8 string primitive bytes
      explicit-at marker: 0x00 normal | 0x01 at-form
      universe_args list in source order as Level canonical bytes
  - Sort:
      level canonical bytes
  - Type:
      level canonical bytes for the displayed Type parameter
  - Prop:
      fixed variant tag
  - App:
      function term canonical bytes
      argument term canonical bytes
  - Lam:
      binders in source order:
        binder local identifier
        binder type term canonical bytes
      body term canonical bytes
  - Pi:
      binders in source order:
        binder local identifier
        binder type term canonical bytes
      body term canonical bytes
  - Let:
      local identifier
      type term canonical bytes
      value term canonical bytes
      body term canonical bytes
  - Annot:
      term canonical bytes
      type term canonical bytes

Level canonical bytes:
  - natural:
      minimal unsigned LEB128 value
  - param:
      universe parameter identifier as UTF-8 string primitive bytes
  - succ:
      payload level canonical bytes
  - max / imax:
      lhs level canonical bytes
      rhs level canonical bytes
```

Whitespace, parentheses that only group expressions, source spans, diagnostic
text, pretty text, and AI traces do not enter term-source canonical bytes.
Binder / local / universe parameter names are encoded as the UTF-8 byte sequence
after JSON / source decode. Therefore alpha-equivalent source that differs only
in binder names may have different Phase 3 term-source hashes. Phase 4
`MachineTermSource.canonical_hash`, which carries that as payload, may also
differ. This conservatively separates cache keys; proof correctness is judged by
the elaborated core term and kernel check.

---

# 10. Connection With Phase 5 / Phase 7

Proof state passed to AI separates human-facing pretty strings from Machine
Surface strings.

```json
{
  "target": {
    "pretty": "n + 0 = n",
    "machine": "Eq.{1} Nat (Nat.add n Nat.zero) n",
    "core_hash": "sha256:...",
    "constants": [
      { "name": "Eq", "decl_interface_hash": "sha256:..." },
      { "name": "Nat.add", "decl_interface_hash": "sha256:..." },
      { "name": "Nat.zero", "decl_interface_hash": "sha256:..." }
    ]
  },
  "locals": [
    {
      "name": "n",
      "type_machine": "Nat",
      "type_hash": "sha256:..."
    }
  ]
}
```

Example AI output:

```json
{
  "tactic": "exact @Eq.refl.{1} Nat n",
  "term_machine": "@Eq.refl.{1} Nat n",
  "expected_target_hash": "sha256:..."
}
```

The Phase 5 tactic execution API checks the term part in Machine Surface
Complete mode. Only terms that pass are passed to tactic execution.

---

# 11. Implementation Plan

The current policy is to keep Human Surface and Machine Surface as separate
profiles, and to fix AI-facing candidate checking to the Machine Surface fast
path. The following milestones are treated as the implementation history from
the AI-first baseline through the Machine Surface M9 gate.

## M0: Restart baseline

Purpose:

```text
Create a clean baseline dedicated to Machine Surface.
```

Work:

```text
- separate Machine Surface APIs from Human Surface APIs
- fix the AI fast-path boundary in develop/phase3-ai.md
- allow README to reference Phase 3 AI / Phase 3 Human separately
- pass cargo test --workspace
```

Completion criteria:

```text
- Machine Surface APIs in crates/npa-frontend are separated from Human Surface APIs
- Phase 1 / Phase 2 tests pass
- the Phase 3 implementation specification is fixed to Machine Surface
```

## M1: Frontend crate skeleton

Purpose:

```text
Create a frontend crate dedicated to Machine Surface.
```

Files to create:

```text
crates/npa-frontend/Cargo.toml
crates/npa-frontend/src/lib.rs
crates/npa-frontend/src/span.rs
crates/npa-frontend/src/diagnostic.rs
crates/npa-frontend/src/machine.rs
crates/npa-frontend/src/lexer.rs
crates/npa-frontend/src/parser.rs
crates/npa-frontend/src/resolver.rs
crates/npa-frontend/src/elaborator.rs
```

Completion criteria:

```text
- npa-frontend is included in the workspace
- public APIs have machine-oriented names
- empty module / simple diagnostic tests pass
```

## M2: Machine parser

Purpose:

```text
Parse only Machine Surface syntax.
```

accepted tests:

```text
- import
- def id
- theorem self_eq
- explicit universe args
- typed fun
- typed forall
- typed let
- annotation
```

rejected tests:

```text
- open
- namespace
- notation declaration
- axiom
- inductive
- hole
- unannotated lambda binder
- unannotated let
- operator notation
```

Completion criteria:

```text
- parser state has no notation table / open scope
- the same input produces the same AST
```

## M3: Direct resolver

Purpose:

```text
Resolve globals only by fully qualified exact match.
```

Work:

```text
- read verified import interfaces
- implement local context lookup
- implement exact global lookup
- reject short global names
- reject local/global shadowing
```

Completion criteria:

```text
- Nat.add can be resolved
- add is rejected
- there is no suffix lookup
- no overload candidates are created
```

## M4: Explicit elaborator

Purpose:

```text
Lower Machine Surface terms to fully explicit core Expr.
```

Work:

```text
- Sort / Type / Prop
- Const / local BVar
- App
- Lam
- Pi
- Let
- annotation
- explicit universe args
- declaration binder closing
```

Completion criteria:

```text
- explicit id becomes a core def
- explicit Eq.refl proof becomes a core theorem
- with a callable profile containing implicit binders, Eq.refl n is ImplicitArgumentRequired
- with an all-explicit callable profile, ImplicitArgumentRequired does not occur due to missing implicit term binders
- fun x => x is UnannotatedBinder
- _ is HoleNotAllowed
```

## M5: Kernel handoff

Purpose:

```text
Create CoreModule from Machine Surface and pass it to the Phase 1 kernel.
```

Work:

```text
- put the imported environment into the kernel env
- check def value : type
- check theorem proof : type
- wrap kernel errors in MachineDiagnostic
```

Completion criteria:

```text
- well-typed defs/theorems pass
- ill-typed applications are rejected
- generated CoreModule does not retain Machine Surface metadata
```

## M6: Certificate integration

Purpose:

```text
Create and verify a Phase 2 certificate from Machine Surface source.
```

Work:

```text
- CoreModule -> build_module_cert
- encode_module_cert
- verify_module_cert
- connection between verified imports and export_hash
```

Completion criteria:

```text
- .npcert can verify without source
- same source gives same certificate hash
- certificates contain no AI trace / source span / Machine metadata
```

## M7: Term-level API for tactics

Purpose:

```text
Phase 5 / Phase 7 can check terms inside tactic candidates as Machine Surface.
```

Work:

```text
- elaborate_machine_term_check
- local context import
- expected type check
- constants / core_hash / contextual_core_hash extraction
```

Completion criteria:

```text
- exact @Eq.refl.{1} Nat n can be checked as a term that closes the goal
- failed terms return structured errors
- tactic execution does not directly trust unchecked AI text
```

## M8: Repair mode

Purpose:

```text
Return structured errors and suggestions that are easy for the AI repair loop to use.
```

Work:

```text
- missing explicit implicit arg suggestion
- short name -> fully qualified candidate suggestion
- missing universe arg suggestion
- type mismatch payload
```

Completion criteria:

```text
- suggestions do not enter trusted payloads
- suggestions are accepted only if resubmitted and passed
- the same failure returns the same error enum
- TypeMismatch returns expected/actual core hash payload
```

## M9: Performance / determinism gate

Purpose:

```text
Confirm stable behavior even when AI search submits many candidates.
```

Measurements:

```text
- parse latency
- resolve latency
- elaborate latency
- failed candidate latency
- allocation
- same input same output
```

The M9 CI gate does not set fixed thresholds for wall-clock latency or
allocation, because they are prone to flakes across execution environments.
Instead, regression tests fix determinism, state independence, and resource
guards so repeated submission of many candidates from AI search does not break
the system.

`same input same output` is limited to the following.

```text
- parsed AST
- term-source canonical bytes / canonical hash
- elaborated term check result
- certificate hash / encoded certificate bytes
- failed candidate diagnostic kind / structured payload / repair suggestions
```

Human-facing diagnostic message strings, wall-clock timing, and allocator
implementation details are not treated as trusted output.

Completion criteria:

```text
- no active notation table exists
- no open scope exists
- no overload transaction exists
- same input returns same parsed AST
- same term-level check input returns same core hash / contextual core hash / constants
- same failed candidate returns same diagnostic kind / structured payload / suggestion
- oversized canonical input is rejected before large allocation
- cargo fmt --all
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
```

---

# 12. MVP Scope

The core frontend MVP is M0 through M6. The current AI fast path is implemented
through the M7 term-level API, M8 repair payload, and M9 determinism /
regression gate needed for Phase 4/5/7 integration.

```text
core M0-M6 includes:
  import
  def
  theorem
  fully qualified names
  explicit universe args
  typed lambda / Pi / let
  kernel handoff
  certificate generation / verification

core M0-M6 excludes:
  axiom source syntax
  inductive source syntax
  notation
  implicit insertion
  holes
  tactic blocks
  repair suggestions

M7-M9 adds:
  term-level Machine Surface API for tactics
  non-trusted repair payload / suggestions
  same-input same-output and resource-guard regressions
```

Nat / Eq / standard theorems are read as Phase 2 verified imports rather than
defined by AI in source.

---

# 13. Completion Criteria

AI-facing Phase 3 can be considered complete when the following conditions hold.

```text
- Machine Surface syntax is implemented
- parser / resolver / elaborator have no notation / open / overload
- there is no implicit term arg insertion
- unresolved holes are unrepresentable or rejected in complete mode
- fully explicit defs/theorems lower to CoreModule
- only terms that pass kernel check are certificate-generated
- .npcert can verify without source
- Phase 5 / Phase 7 can use the term-level API
- structured errors are usable for AI repair
- cargo fmt / clippy / test pass
```

---

# 14. In One Sentence

AI-facing Phase 3 is **not a human-convenient surface language, but a Machine
Surface frontend that quickly and deterministically lowers the many candidates
generated by AI to canonical core AST**.

Convenient syntax can be added later as a separate profile. In the MVP, only
near-fully-explicit syntax is accepted and passed to the kernel and certificate
verifier by the shortest path.
