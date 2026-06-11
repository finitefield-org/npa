This document is the detailed design for **Phase 3 Human Profile: Human Surface
Language**. The Phase 1/2 kernel and certificate layers accept only fully
explicit core AST. Phase 3 adds human-writable syntax and lowers it to
**canonical core AST**.

Implementation status (2026-05-21):

```text
- The Human Surface MVP is implemented in crates/npa-frontend as the human parser / resolver / elaborator / certificate handoff.
- The AI-facing path is separated into the Machine Surface fast path in develop/phase3-ai.md.
- Human Surface parser / resolver / elaborator / metadata are convenience layers and are not trusted base.
```

Core premise:

```text
parser / names / notation / implicit args / holes / elaborator
are convenient untrusted layers.

Final correctness is checked by
the canonical certificate,
the Phase 1 Rust kernel,
the Phase 2 certificate verifier,
and the Phase 8 independent checker.
```

---

# 1. Purpose of Phase 3

Phase 3 input is human-facing source.

```npa
namespace Nat

def add (n m : Nat) : Nat :=
  rec (fun _ => Nat) n (fun _ ih => succ ih) m

theorem add_zero (n : Nat) : add n zero = n :=
  Eq.refl n

end Nat
```

It is converted into the Phase 1/2 representation.

```text
surface source
  ↓ parser with active notation table
surface AST with notation nodes
  ↓ name resolution + notation candidate collection
resolved AST with overloaded names / notation candidates
  ↓ elaboration
elaboration AST + metavariables/goals
  ↓ metavariable solving
fully explicit core AST
  ↓ certificate generation
canonical certificate
```

Phase 3 succeeds when:

```text
- humans can write simple def/theorem/axiom/simple inductive declarations
- import / namespace / open and name resolution work
- infix notation and similar syntax can be used
- omitted arguments can be inserted
- `_` and `?m` can create holes
- simple elaboration can produce core terms
- unresolved holes are displayed as goals and never emitted into certificates
```

`elaboration AST` has the same logical structure as core terms, but may
temporarily contain metavariables and untrusted metadata during elaboration. For
example, implicit insertion may attach `BinderInfo` to a `Pi` spine. The
`fully explicit core AST` passed to Phases 1/2 contains no metavariables, holes,
notation, or implicit-argument metadata.

Examples in this document may omit universe arguments for readability. For
example, `Eq Nat n n` is explanatory display; fully explicit core / certificate
form must spell the universe level, such as `Eq.{1} Nat n n`.

---

# 2. Parser

## 2.1 Parser Responsibility

The parser converts text into a surface AST.

Parser responsibilities:

```text
- lexing
- parenthesis structure
- declaration syntax
- binder syntax
- term syntax
- notation syntax
- source locations
```

Not parser responsibilities:

```text
- name resolution
- type inference
- implicit argument inference
- typeclass search
- theorem search
- kernel check
```

For example, when the parser sees:

```npa
x + y = x
```

it does not decide whether `+` is Nat addition, Int addition, or another
operation. That is elaboration's job.

## 2.2 Minimum Syntax

The Phase 3 MVP needs only:

```text
module
  import
  open
  namespace
  end
  notation declaration
  def
  theorem
  axiom
  simple inductive

term
  identifier
  explicit universe application
  Sort
  Prop
  Type
  application
  lambda
  forall / Pi
  let
  annotation
  hole
  parenthesized term
  non-dependent arrow
  simple notation
```

Grammar sketch:

```text
item ::=
    "import" qual_name
  | "open" qual_name
  | "namespace" name
  | "end" name?
  | notation_decl
  | def_decl
  | theorem_decl
  | axiom_decl
  | inductive_decl

def_decl ::=
    "def" name universe_params? decl_binder* ":" term ":=" term

theorem_decl ::=
    "theorem" name universe_params? decl_binder* ":" term ":=" term

axiom_decl ::=
    "axiom" name universe_params? decl_binder* ":" term

inductive_decl ::=
    "inductive" name universe_params? decl_binder* ":" term "where" ctor_decl+

ctor_decl ::=
    "|" name ":" term

notation_decl ::=
    ("prefix" | "postfix" | "infix" | "infixl" | "infixr")
    ":" precedence string "=>" qual_name

decl_binder ::=
    "(" ident+ ":" term ")"
  | "{" ident+ ":" term "}"

lambda_binder ::=
    decl_binder
  | ident
  | "_"

pi_binder ::=
    decl_binder

term ::=
    ident universe_args?
  | "@" ident universe_args?
  | "Prop"
  | "Type" level?
  | "Sort" level
  | term term
  | "fun" lambda_binder+ "=>" term
  | "forall" pi_binder+ "," term
  | "let" ident ":" term ":=" term "in" term
  | "let" ident ":=" term "in" term
  | term ":" term
  | term "->" term
  | term "→" term
  | "_"
  | "?" ident
  | "(" term ")"

universe_params ::= ".{" ident ("," ident)* "}"
universe_args   ::= ".{" level ("," level)* "}"

level ::=
    natural
  | ident
  | "succ" level
  | "max" level level
  | "imax" level level
```

`import` may appear only at the beginning of a module. An import after the first
non-import item is rejected as `ImportAfterItem`, because imports affect the
notation table and global scope.

`namespace`, `open`, and `notation` are ordinary items processed top-to-bottom
and affect only later items.

In the MVP, declaration binders and `forall` binders require type annotations.
Only lambda binders such as `fun x => ...` or `fun _ => ...` may omit
annotations, and only in check mode with an expected type. Numeric literals and
typeclass-driven overloads are not Phase 3 features; write natural zero as
`Nat.zero` or `zero` from an opened namespace.

`_` in lambda binder position is an anonymous binder, not a term hole. It adds a
local binder with display name `_`, but the binder cannot be referenced by name.

`A -> B` and `A → B` are built-in right-associative syntax that desugars to
`Pi _ : A, B`. They are reserved and cannot be redefined as user notation.

Term precedence is fixed in the MVP:

```text
highest
  atom / parenthesized term / identifier universe args
  explicit application, left associative
  prefix notation
  postfix notation
  infix notation, using notation-table precedence and associativity
  type annotation `:`, non-associative
  arrow `->` / `→`, right associative
  forall / lambda / let body
lowest
```

Two active notation entries with the same symbol may coexist as overloads only
when kind, precedence, and associativity all match. Otherwise reject them as
`NotationConflict`.

`inductive` is limited to shapes that lower to the Phase 1/2 `simple inductive`.
Mutual / nested / coinductive declarations, pattern matching syntax, and user
macros are outside the Phase 3 MVP.

## 2.3 Surface AST

The parser emits surface AST, not core AST.

```rust
enum ImplicitMode {
    Insert,
    Explicit,
}

enum SurfaceBinderKind {
    Named(SurfaceName),
    Anonymous,
}

struct SurfaceBinder {
    kind: SurfaceBinderKind,
    ty: Option<Box<SurfaceExpr>>,
    binder_info: BinderInfo,
    span: Span,
}

enum SurfaceExpr {
    Ident {
        name: SurfaceName,
        universe_args: Option<Vec<SurfaceLevel>>,
        implicit_mode: ImplicitMode,
        span: Span,
    },
    Sort {
        level: SurfaceLevel,
        span: Span,
    },
    App {
        func: Box<SurfaceExpr>,
        arg: Box<SurfaceExpr>,
        span: Span,
    },
    Lam {
        binders: Vec<SurfaceBinder>,
        body: Box<SurfaceExpr>,
        span: Span,
    },
    Pi {
        binders: Vec<SurfaceBinder>,
        body: Box<SurfaceExpr>,
        span: Span,
    },
    Let {
        name: SurfaceName,
        ty: Option<Box<SurfaceExpr>>,
        value: Box<SurfaceExpr>,
        body: Box<SurfaceExpr>,
        span: Span,
    },
    Annot {
        expr: Box<SurfaceExpr>,
        ty: Box<SurfaceExpr>,
        span: Span,
    },
    Hole {
        name: Option<SurfaceName>,
        span: Span,
    },
    Notation {
        head: NotationHead,
        args: Vec<SurfaceExpr>,
        span: Span,
    },
}
```

`ImplicitMode` is normally `Insert`. It becomes `Explicit` only for heads such
as `@Eq.refl`. An `Explicit` head does not auto-insert implicit term arguments;
user-supplied positional arguments are consumed from left to right. This is
application-head mode, separate from binder `BinderInfo::Explicit`.

`SurfaceBinder` is shared by declaration, lambda, and Pi binders.

```text
(x : A)  -> kind = Named(x), ty = Some(A), binder_info = Explicit
{x : A}  -> kind = Named(x), ty = Some(A), binder_info = Implicit
x        -> kind = Named(x), ty = None,    binder_info = Explicit
_        -> kind = Anonymous, ty = None,   binder_info = Explicit
```

`(x y : A)` and `{x y : A}` expand left-to-right into multiple binders. The
annotation `A` is elaborated before `x` / `y` enter scope, so `(x y : A)` means
`(x : A) (y : A)`. Use `(x : A) (y : B x)` for dependency.

Declaration binders and `forall` / Pi binders forbid `ty = None`. Only
`fun x => ...` and `fun _ => ...` may omit binder types. Arrow syntax desugars
to `SurfaceExpr::Pi` with an anonymous binder.

When lambda source uses `{x : A}`, infer mode marks the inferred Pi spine with
`Implicit`. In check mode, the expected Pi's `BinderInfo` takes priority. MVP
may warn on mismatch; core lowering erases both.

`Prop`, `Type`, and `Type u` may be normalized by the parser:

```text
Prop   -> Sort 0
Type   -> Sort 1
Type u -> Sort (succ u)
```

`Span` is required for IDE and error display.

```rust
struct Span {
    file_id: FileId,
    start: ByteOffset,
    end: ByteOffset,
}
```

Source maps are not trusted certificate payload. Even if packaged as debug
sections, they do not affect kernel checks, canonical hashes, or axiom reports.

`SurfaceExpr::Notation` records only that precedence and associativity were
settled. The target constant may still be unresolved. The resolver attaches
`ElabGlobalRef` candidates from active notation entries, and the elaborator uses
types to choose one.

## 2.4 Front-End State and Processing Order

The parser / resolver / elaborator processes each module top-to-bottom so
notation and namespaces affect only later source.

Untrusted front-end state includes:

```rust
struct FrontendState {
    current_module: ModuleName,
    namespace_stack: Vec<Name>,
    open_scopes: Vec<OpenScope>,
    globals: GlobalScope,
    locals: LocalScopeStack,
    notation_table: NotationTable,
    source_interfaces: SourceInterfaceStore,
}
```

`current_module` comes from the compile request or package manifest. `import` is
untrusted resolution from module name to source interface / verified export. The
kernel does not resolve imports.

Source `import M` writes only the module name. Phase 3 finds `M` in the compile
request's `verified_imports`. Missing imports, or multiple hashes for the same
module name, are `ImportResolutionError`. The MVP has no hash literal syntax in
`.npa` source.

Repeated import of the same module and same hash may warn and normalize to one
entry. Same module name with different hashes is `ImportResolutionError`.

MVP module processing:

```text
1. process imports and add imported declarations / source interface metadata
2. add imported top-level notation metadata to the active notation table
3. parse / resolve / elaborate items top-to-bottom
4. namespace / open / notation affect only later items
5. def/theorem/axiom/inductive become visible only after kernel check succeeds
6. at module end, pass only complete declarations to the Phase 2 certificate producer
```

Source interface metadata may contain display names, `BinderInfo`, notation,
source spans, and docs. It is not trusted certificate payload. Wrong metadata
can at most influence convenience; the final explicit core declaration is
checked by Phases 1/2.

`open N` is accepted only when prefix `N` exists among visible declarations /
notation. Opening an unknown namespace is `UnknownNamespace`. Empty namespace
declarations are not in the MVP.

Declarations containing unresolved holes may return goals in interactive mode,
but are not registered as trusted dependencies. The MVP rejects later references
to incomplete declarations with `IncompleteDependency`.

---

# 3. Names

## 3.1 Name Kinds

Names are hierarchical from the start.

```text
Nat
Nat.zero
Nat.succ
Nat.add
Eq
Eq.refl
Algebra.Group.mul_assoc
```

Internal shape:

```rust
struct Name(Vec<NamePart>);

enum NamePart {
    Str(String),
    Num(u32),
}
```

Future implementations may intern names as `NameId`.

## 3.2 Local Names and Global Names

Two major categories:

```text
local name:
  introduced by lambdas or theorem binders

global name:
  declaration defined by def/theorem/axiom/inductive
```

Example:

```npa
theorem t (Nat : Type) (x : Nat) : x = x :=
  Eq.refl x
```

Here local `Nat` shadows global `Nat`. The MVP may allow this but should warn.

```text
warning:
  local name `Nat` shadows global declaration `Nat`
```

## 3.3 Name Resolution Order

Term-name resolution is separate from declaration-name qualification. Local
names can be referenced only by unqualified identifiers. A qualified name such
as `Nat.x` never refers to a local binder.

Source `qual_name` refers to visible declaration names, not module names. If
module `Std.Nat.Basic` exports declaration `Nat.add`, source refers to `Nat.add`,
not `Std.Nat.Basic.Nat.add`. Module names are used for import resolution and
certificate import tables.

Unqualified `add` resolves by:

```text
1. local context
2. current namespace + add
3. opened namespaces + add
4. root/imported short name add
```

Qualified `Nat.add` resolves by:

```text
1. visible global declaration exactly named Nat.add
2. current namespace + Nat.add
3. opened namespaces + Nat.add
4. imported declaration whose declaration name has suffix Nat.add
```

Within `namespace Nat`, `Nat.add` first looks for exact global `Nat.add`, which
lets code refer stably to the current namespace by qualified name.

`open Nat` is untrusted elaboration metadata for the current lexical scope. If
the same short name comes from multiple opened/imported namespaces, report
ambiguity instead of guessing. Aliases are not in the MVP.

If one candidate exists at a priority level, use it. If multiple exist at the
same priority, report ambiguity. If a higher priority has candidates, ignore
lower priorities. Current-module checked declarations take priority over
imported declarations; multiple imports exporting the same declaration name are
ambiguous unless a current-module declaration shadows them.

## 3.4 Ambiguity

The same short name may refer to:

```text
Nat.add
Int.add
Rat.add
```

Policy:

```text
- if unique without type information, resolve immediately
- if multiple candidates exist, keep an OverloadedRef
- try to resolve during elaboration from expected type
- fail if still unresolved
```

Intermediate representation:

```rust
enum ResolvedName {
    Local(LocalId),
    Global(ElabGlobalRef),
    Overloaded(Vec<ElabGlobalRef>),
    Unresolved(SurfaceName),
}
```

## 3.5 ElabGlobalRef

To connect with Phase 2 certificates, global references are hash-bound, not only
name-based.

```rust
enum ElabGlobalRef {
    Imported {
        module: ModuleName,
        name: NameId,
        decl_interface_hash: Hash,
    },
    Local {
        decl_index: usize,
        name: NameId,
    },
    LocalGenerated {
        decl_index: usize,
        name: NameId,
    },
}
```

This prevents two different declarations named `Nat.add` from being confused.

This is a source/elaboration-layer reference. Imported declarations are fixed by
the `decl_interface_hash` in the import `export_hash`. Current-module
declarations are referenced by local declaration index after kernel check.
Generated declarations such as inductive constructors / recursors use
`LocalGenerated`.

Before Phase 2 handoff, these lower to canonical `GlobalRef::Imported`,
`GlobalRef::Local`, and `GlobalRef::LocalGenerated`. Module names or display
names are never the trusted basis.

The inductive head being elaborated may be a temporary global before kernel
check. It is used only for constructor type elaboration. After the whole
`InductiveDecl` passes kernel check, temporary refs are replaced with ordinary
`Local` / `LocalGenerated`.

## 3.6 Scope Details

MVP scope rules:

```text
namespace N:
  push N onto namespace_stack

end N:
  pop if the stack tail is N
  if N is omitted, pop the tail
  otherwise NamespaceMismatch

open N:
  add N to the current lexical scope
```

`open` does not leak outside a namespace block. Entering a namespace pushes an
open-scope frame; the matching `end` pops it. Top-level `open` lasts to module
end.

`open` affects both names and imported/exported notation metadata. After
`open Nat`, notation entries belonging to namespace `Nat` become active. The
same notation conflict rules apply.

Declaration names are:

```text
module_name = current_module
declaration_name = namespace_stack + declaration_name
```

Qualified source declaration names are relative to the current namespace in the
MVP. Inside `namespace Nat`, `def Extra.double` becomes `Nat.Extra.double`.
Absolute-name syntax is not in the MVP.

Duplicate global declaration names in one module are forbidden. Names generated
by simple inductives, such as constructors and recursors, are in the same global
scope and can cause `DuplicateDeclaration`.

Local shadowing is safe with de Bruijn indices and is allowed; nearest binder
wins. Shadowing should produce warnings for IDE/API but not affect trusted
payload.

Notation declarations record the current namespace as metadata. In the current
file, they are active only after their declaration and within the current
lexical namespace frame. Imported namespaced notation is active when that
namespace is entered or opened.

---

# 4. Notation

## 4.1 Purpose

Humans want to write:

```npa
n + Nat.zero = n
```

Core is:

```text
Eq Nat (Nat.add n Nat.zero) n
```

Notation bridges surface syntax and core constants.

## 4.2 Start Small

The MVP supports:

```text
prefix
infix
postfix
```

Examples:

```npa
infixl:65 " + " => Nat.add
infix:50 " = " => Eq
```

Because `=` has implicit type argument `A`, surface:

```text
x = y
```

expands conceptually to:

```text
Eq ?A x y
```

and elaboration infers `?A`.

## 4.3 Notation Table

```rust
struct NotationEntry {
    symbol: String,
    kind: NotationKind,
    precedence: u16,
    associativity: Associativity,
    target: NotationTarget,
    namespace: Option<Name>,
}

enum NotationKind {
    Prefix,
    Infix,
    Postfix,
}

enum Associativity {
    Left,
    Right,
    NonAssoc,
}

enum NotationTarget {
    Global(ElabGlobalRef),
}
```

Phase 3 notation expands only to `ElabGlobalRef`. Syntax macros and user syntax
extensions are not in the MVP.

The target `qual_name` of a notation declaration is resolved when the notation
declaration is processed. Undefined or overloaded targets are rejected.
Notation cannot be forward-declared and cannot target local binders or temporary
globals.

`prefix` and `postfix` are always `NonAssoc`; `infix` is `NonAssoc`, `infixl` is
`Left`, and `infixr` is `Right`.

## 4.4 Parser and Notation

Use a Pratt parser or precedence climbing for infix notation.

```npa
a + b * c = d
```

parses according to precedence into a notation tree corresponding to:

```text
Eq (Nat.add a (Nat.mul b c)) d
```

Recommended:

```text
- use a Pratt parser
- pass the notation table to the parser
- leave type-based notation resolution to the elaborator
```

Chaining a non-associative infix at the same precedence is a parse error.

```npa
a = b = c
```

with `=` declared as `infix:50` is `ParserError`; write parentheses if needed.

Notation declarations have fixed syntax and can be parsed without the notation
table. Imported notation metadata is convenience metadata and is not trusted
certificate payload.

## 4.5 Overloaded Notation

`+` may mean:

```text
Nat.add
Int.add
Rat.add
Group.add
```

Phase 3 keeps only finite overload candidates produced by the notation table /
name resolution. No typeclass search or algebraic hierarchy search happens in
Phase 3.

```rust
SurfaceExpr::Notation {
    head: "+",
    args: [a, b],
}
```

may resolve to:

```rust
ResolvedExpr::OverloadedApp {
    candidates: [Nat.add, Int.add, Rat.add],
    args: [a, b],
}
```

Elaboration selects by expected type and argument types.

## 4.6 Notation Does Not Remain in Certificates

```text
source:
  n + Nat.zero = n

certificate:
  Eq Nat (Nat.add n Nat.zero) n
```

Notation is entirely untrusted.

## 4.7 Notation Parsing Decisions

Lex notation symbols by longest match. If active symbols include `<` and `<=`,
input `<=` is one token. Symbols are UTF-8 strings but never core names or hash
inputs.

The notation string must trim to one operator token in the MVP. `" + "` and
`"+"` are the same symbol. Multi-token and mixfix notation are deferred.

Reserved tokens cannot be notation symbols:

```text
->, →, :, :=, =>, ,, ., .{, (, ), {, }, |, @, _, ?
```

Notation processing has two stages:

```text
parser:
  use only precedence / associativity and produce SurfaceExpr::Notation

resolver / elaborator:
  use notation table and name resolution to build ElabGlobalRef candidates, then select by type
```

Candidate order is deterministic:

```text
1. display fully qualified declaration name by UTF-8 byte lexicographic order
2. ref kind order: Local, LocalGenerated, Imported
3. Local / LocalGenerated by decl_index and generated name
4. Imported by module name and decl_interface_hash bytes
```

Each overload candidate uses its own metavariable / constraint transaction.
Failed candidates do not leak assignments or constraints. If multiple
candidates succeed, report ambiguity instead of picking the first.

---

# 5. Implicit Args

## 5.1 Purpose

Core arguments are explicit:

```text
Eq.refl.{1} Nat n
```

Humans want:

```npa
Eq.refl n
```

or:

```npa
refl
```

The elaborator inserts omitted arguments.

## 5.2 BinderInfo

Declarations carry explicitness as elaboration metadata.

```rust
enum BinderInfo {
    Explicit,
    Implicit,
}
```

This metadata is for implicit insertion and is not part of canonical core terms
or certificate hashes. `StrictImplicit` and `InstanceImplicit` are future
features.

Example:

```npa
Eq.{u} {A : Sort u} (x : A) (y : A) : Prop
Eq.refl.{u} {A : Sort u} (x : A) : x = x
```

Imported declaration `BinderInfo` comes from untrusted source interface
metadata attached to declaration hashes. If metadata is wrong, the resulting
explicit core term is still checked by the kernel / checker.

## 5.3 Implicit Insertion

For:

```npa
Eq.refl n
```

`Eq.refl` conceptually has type:

```text
Π {A : Sort u}, Π x : A, Eq A x x
```

The elaborator inserts:

```text
Eq.refl ?A n
```

and from `n : Nat` solves:

```text
?A := Nat
```

Final core:

```text
Const Eq.refl [1] Nat n
```

## 5.4 Algorithm

When elaborating application, WHNF the function type.

```text
f_ty = Π binder, body
```

Then:

```text
if binder is implicit and insertion applies:
  create metavariable and insert it

if binder is implicit and `@` mode:
  consume a user argument if one exists

if binder is explicit:
  consume a user argument
```

Pseudocode:

```rust
fn elaborate_app(func: SurfaceExpr, args: Vec<SurfaceExpr>, expected: Option<ElabExpr>)
    -> Result<(ElabExpr, ElabExpr)>
{
    let auto_insert = implicit_insertion_enabled_for_head(&func);
    let (mut f_core, mut f_ty) = elaborate_infer(func)?;
    let mut remaining_args = args.into_iter().peekable();

    loop {
        let f_ty_whnf = whnf(f_ty);

        match f_ty_whnf {
            Pi { binder_info: Implicit, domain, body }
                if auto_insert
                    && (remaining_args.peek().is_some()
                        || expected_needs_implicit_instantiation(&f_ty_whnf, expected.as_ref())) =>
            {
                let m = fresh_meta(domain);
                f_core = mk_app(f_core, m);
                f_ty = instantiate(body, m);
            }

            Pi { binder_info: Implicit, domain, body } if !auto_insert => {
                let Some(arg) = remaining_args.next() else {
                    break;
                };

                let arg_core = elaborate_check(arg, domain)?;
                f_core = mk_app(f_core, arg_core);
                f_ty = instantiate(body, arg_core);
            }

            Pi { binder_info: Implicit, .. } => {
                break;
            }

            Pi { binder_info: Explicit, domain, body } => {
                let Some(arg) = remaining_args.next() else {
                    break;
                };

                let arg_core = elaborate_check(arg, domain)?;
                f_core = mk_app(f_core, arg_core);
                f_ty = instantiate(body, arg_core);
            }

            _ => break,
        }
    }

    if remaining_args.peek().is_some() {
        return Err(Error::TooManyArguments);
    }

    Ok((f_core, f_ty))
}
```

`expected_needs_implicit_instantiation` may be conservative. If unclear, return
false and later report `UnsolvedImplicit` or `TypeMismatch`.

`implicit_insertion_enabled_for_head` returns false only for `@ident` heads.

## 5.5 Explicit Implicit Arguments

Phase 3 supports `@` to turn off automatic insertion of implicit term args.

```npa
@Eq.refl Nat n
```

```text
Eq.refl n
  elaborates to Eq.refl ?A n

@Eq.refl Nat n
  elaborates to Eq.refl.{?u} Nat n
  and then ?u := 1
```

`@` stops automatic insertion of implicit **term** arguments only. It does not
stop universe metavariable generation. Fully explicit form is:

```npa
@Eq.refl.{1} Nat n
```

Named arguments such as `Eq.refl {A := Nat} n` are future work. The MVP only
needs `@`.

## 5.6 Implicit Insertion Boundary

MVP rules:

```text
- reading a bare identifier in infer mode does not insert implicit args
- application heads insert implicit args only when needed to consume the next user argument
- check mode may insert implicit args until the expected type can match
- `@` heads do not auto-insert implicit term args
- remaining implicit binders after explicit user args are not inserted unless required by the expected type
```

If imported declarations lack `BinderInfo`, treat all binders as explicit. This
is conservative but less convenient.

`SyntheticImplicit` metavariables are normally not shown as user goals. If they
remain at declaration close, report `UnsolvedImplicit`. `UserHole`
metavariables may be shown as interactive goals, but certificate generation
rejects them if unsolved.

---

# 6. Holes

## 6.1 Purpose

Holes represent incomplete terms.

```npa
theorem add_zero (n : Nat) : n + Nat.zero = n :=
  _
```

or:

```npa
theorem add_zero (n : Nat) : n + Nat.zero = n :=
  ?proof
```

Phase 3 turns holes into metavariables.

```text
_      -> fresh metavariable
?proof -> named metavariable
```

## 6.2 Holes Never Enter Certificates

```text
A theorem with unresolved holes cannot become a certificate.
```

Development may allow holes.

```text
interactive elaboration:
  holes allowed

certificate generation:
  holes forbidden
```

This forbids **unsolved** holes. A `UserHole` produced from `_` or `?m` may be
solved uniquely by elaboration / unification; then it is lowered to normal core.
Only unsolved `UserHole` values become interactive goals.

Incomplete response:

```json
{
  "status": "incomplete",
  "goals": [
    {
      "name": "?proof",
      "context": [
        {"name": "n", "type": "Nat"}
      ],
      "target": "n + Nat.zero = n"
    }
  ]
}
```

## 6.3 Metavariable

```rust
struct MetaVar {
    id: MetaVarId,
    name: Option<NameId>,
    context: LocalContextSnapshot,
    ty: ElabExpr,
    assignment: Option<ElabExpr>,
    kind: MetaVarKind,
    span: Span,
}

enum MetaVarKind {
    UserHole,
    SyntheticImplicit,
    UniverseMeta,
}
```

`ElabExpr` exists only inside the elaborator. It is mostly like `CoreExpr`, but
may contain metavariable refs and untrusted metadata such as `BinderInfo`.
Before certificate generation, every `ElabExpr` must lower to metavariable-free
`CoreExpr`.

The context snapshot matters. For:

```npa
theorem t (n : Nat) : n = n := _
```

the goal is:

```text
n : Nat
⊢ n = n
```

## 6.4 Named Holes

Policy:

```text
?m is the same metavariable within the same scope
_ is a fresh metavariable every time
```

```npa
(?m, ?m)
```

requires both occurrences to be the same value.

```npa
(_, _)
```

creates two independent holes.

## 6.5 Hole Display

IDE/API returns structured goals.

```json
{
  "hole": "?proof",
  "context": [
    {
      "name": "n",
      "type": "Nat"
    }
  ],
  "target": "Eq Nat (Nat.add n Nat.zero) n",
  "source_span": {
    "line": 3,
    "column": 3
  }
}
```

This feeds the Phase 4 tactic API.

## 6.6 Named Hole Reuse and Context

`?m` reuse is scoped to one declaration. `?m` in different declarations is a
different metavariable.

Within one declaration, the MVP rejects reuse of the same named hole under
different local context snapshots.

```npa
def bad : Nat :=
  let x : Nat := ?m in ?m
```

MVP does not implement higher-order context abstraction for metavariables.

```text
same named hole + same context snapshot:
  return the same MetaVarId

same named hole + different context snapshot:
  NamedHoleContextMismatch
```

Context comparison uses local declaration length, types, and local definition
values as core/ElabExpr structure. Display names and source spans are ignored.

Metavariable assignments must typecheck in the metavariable context snapshot.
Assignments perform occurs check; `?m := ... ?m ...` is rejected.

---

# 7. Simple Elaboration

## 7.1 Responsibility

Elaboration converts surface/resolved expressions to `ElabExpr`, and only when
all metavariables are solved lowers them to canonical `CoreExpr`.

Responsibilities:

```text
- convert resolved names to core Const / BVar
- choose notation candidates and turn them into core constant applications
- insert implicit args
- convert holes to metavariables
- infer/check types using expected types
- solve metavariables with simple unification
- solve universe metavariables
```

Non-goals:

```text
- complex typeclass search
- tactic execution
- AI completion
- advanced coercions
- full overloaded algebraic hierarchy resolution
- source declaration reordering
- forward reference resolution
```

Phase 3 is simple elaboration. More powerful conveniences belong to later
phases.

## 7.2 Bidirectional Elaboration

Use two modes.

```text
infer mode:
  infer a type from an expression

check mode:
  check an expression against an expected type
```

API:

```rust
fn elab_infer(expr: SurfaceExpr) -> Result<(ElabExpr, ElabExpr)>;

fn elab_check(expr: SurfaceExpr, expected: ElabExpr) -> Result<ElabExpr>;
```

`fun x => x` needs an expected type unless `x` has an annotation. Lambda is
primarily check-mode; application is primarily infer-mode.

## 7.3 Basic Rules

### Sort

```npa
Prop
Type
Type 0
Type u
Sort u
```

lower to core `Sort`.

```text
Prop   -> Sort 0
Type   -> Sort 1
Type 0 -> Sort 1
Type 1 -> Sort 2
Type u -> Sort (succ u)
```

### Identifier

Resolved identifiers lower to:

```text
local x
  -> BVar i

global Nat.add
  -> Const Nat.add levels
```

Universe-polymorphic global declarations create universe metavariables such as
`id.{?u}` internally.

### Application

```npa
f a
```

Elaboration:

```text
elab f
whnf type(f)
insert implicit args
check a against the domain
return instantiated codomain
```

### Lambda

Given expected type:

```text
Π x : A, B
```

source:

```npa
fun x => body
```

produces:

```text
Lam A body_core
```

Without expected type, a binder annotation is required.

### Pi / forall

```npa
forall (x : A), B
```

lowers to:

```text
Pi A B
```

Check `A : Sort u` and `B : Sort v`, then return `Sort (imax u v)`.

### Let

```npa
let x : A := v in body
```

lowers to:

```text
Let A v body
```

If the annotation is absent, infer the type of `v`.

### Annotation

```npa
(t : T)
```

means:

```text
elaborate T as a type
check t against T
return t_core : T_core
```

## 7.4 Metavariable Solving

Implicit args and holes create metavariables.

```npa
Eq.refl n
```

creates:

```text
Eq.refl ?A n
```

with constraint:

```text
n : ?A
```

If `n : Nat`, solve:

```text
?A := Nat
```

Phase 3 unification is conservative.

Supported:

```text
- ?m := term
- type matching through definitional equality
- simple first-order application matching
- universe metavariable solving
```

Not supported:

```text
- full higher-order pattern unification
- typeclass search
- coercion search
- backtracking-heavy overload resolution
```

Metavariable policy:

```text
SyntheticImplicit:
  auto-assign if unification solves it
  otherwise UnsolvedImplicit at declaration close

UniverseMeta:
  auto-assign if universe constraint solving solves it
  otherwise UnsolvedUniverseMeta at declaration close

UserHole:
  may auto-assign if expected type / constraints determine it uniquely
  otherwise display as interactive goal and reject certificate generation with UnsolvedHole
```

For example, `Eq.refl _` checked against `n = n` can solve `_ := n`. But
`theorem t : Nat := _` leaves a goal.

## 7.5 Expected Type Propagation

Expected types help solve implicits and notation.

```npa
theorem t (n : Nat) : n = n :=
  Eq.refl n
```

Expected type:

```text
Eq.{1} Nat n n
```

RHS:

```text
Eq.refl ?A n
```

Expected type gives:

```text
?A := Nat
```

## 7.6 Overload Resolution

Resolve overloaded notation/names from expected type and argument types. Do not
use typeclass search or deep backtracking in Phase 3.

Strategy:

```text
1. try each candidate
2. collect candidates that typecheck
3. zero successes -> error
4. one success -> use it
5. multiple successes -> ambiguity error
```

Candidate trials are transactions. Failed candidates do not leak metavariables
or constraints. In interactive mode, a candidate that leaves a new unsolved
`UserHole` is pending, not complete success. In certificate mode, any candidate
that leaves unsolved metavariables fails.

## 7.7 Local Context and de Bruijn Lowering

Elaboration may store local names, but core lowering converts them to de Bruijn
indices.

```rust
struct LocalEntry {
    name: NameId,
    ty: ElabExpr,
    value: Option<ElabExpr>,
    binder_info: BinderInfo,
    span: Span,
}
```

Lookup searches from the end. The innermost binder is `BVar 0`.

```text
context:
  A : Type
  x : A

surface `x` -> BVar 0
surface `A` -> BVar 1
```

Source binder order is preserved when closing `Pi` / `Lam`.

```npa
def id (A : Type) (x : A) : A := x
```

lowers to:

```text
type  = Pi A : Sort 1, Pi x : BVar 0, BVar 1
value = Lam A : Sort 1, Lam x : BVar 0, BVar 0
```

`BinderInfo` remains only in source interface metadata.

## 7.8 Constraint Store

Elaboration may accumulate constraints.

```rust
enum Constraint {
    IsType(ElabExpr),
    TypeEq { lhs: ElabExpr, rhs: ElabExpr },
    TermEq { ty: ElabExpr, lhs: ElabExpr, rhs: ElabExpr },
    LevelEq { lhs: ElabLevel, rhs: ElabLevel },
    LevelLe { lhs: ElabLevel, rhs: ElabLevel },
}
```

Solve constraints with a deterministic worklist ordered by source span, stable
generation id, and constraint kind. Resource-limit exhaustion rejects complete
declaration generation.

Overload/notation/expected-type branches use transactions. Uncommitted
transactions discard assignments, constraints, and warnings.

## 7.9 Universe Parameter Policy

Phase 3 MVP requires declaration-level universe polymorphism to be explicit.

```npa
def id.{u} {A : Sort u} (x : A) : A := x
```

Only explicit `.{...}` parameters enter `universe_params`. A `Sort u` or
`Type u` using undeclared `u` is `UnknownUniverseParam`.

Universe parameter names are separate from term/local/global names. Duplicate
universe parameter names in one declaration are forbidden. Term binders with the
same display name should warn.

`Type` and `Type 0` are both `Sort 1`, so:

```npa
def id {A : Type} (x : A) : A := x
```

is monomorphic at `A : Sort 1`.

Using polymorphic constants may create internal universe metavariables. At
declaration close, the MVP does not auto-generalize unresolved universe metas;
it reports `UnsolvedUniverseMeta` and asks for explicit `.{u}` / `Sort u`.

## 7.10 Core Lowering Conditions

`ElabExpr` can lower to `CoreExpr` only when:

```text
- no UserHole / SyntheticImplicit / UniverseMeta remains
- no OverloadedRef / OverloadedApp remains
- every local reference is converted to de Bruijn index
- every global reference has fixed ElabGlobalRef and universe args
- source-only BinderInfo / Span / notation head is removed
- generated core declaration is accepted by the Phase 1 kernel
```

Failed declarations are not emitted to `.npcert`. Interactive APIs return
incomplete/error status and structured diagnostics.

---

# 8. Declaration Elaboration

## 8.1 def

Surface:

```npa
def id {A : Type} (x : A) : A := x
```

After elaboration:

```text
name:
  id

universe params:
  []; MVP does not auto-generalize unresolved universe metas

type:
  Π {A : Sort 1}, Π x : A, A

value:
  λ A : Sort 1, λ x : A, x

reducibility:
  reducible
```

`{A : Type}` is an implicit source binder. The displayed `Π {A : Sort 1}` is a
source interface view. Core erases binder info.

## 8.2 theorem

Surface:

```npa
theorem self_eq (n : Nat) : n = n :=
  Eq.refl n
```

Elaboration:

```text
type:
  Π n : Nat, Eq.{1} Nat n n

proof:
  λ n : Nat, Eq.refl.{1} Nat n
```

The kernel checks:

```text
proof : type
```

## 8.3 axiom

Surface:

```npa
axiom funext :
  ...
```

Only the type is elaborated.

```text
no value/proof
appears in axiom report
```

High-trust mode rejects non-allowlisted axioms.

## 8.4 simple inductive

Surface:

```npa
inductive Nat : Type where
| zero : Nat
| succ : forall (n : Nat), Nat
```

Lower to Phase 1/2 `InductiveDecl`.

```text
name:
  Nat

universe params:
  explicit only

params / indices:
  declaration binders become params
  leading binders in the type after `:` become indices

sort:
  Type

constructors:
  Nat.zero : Nat
  Nat.succ : forall (n : Nat), Nat
```

Phase 3 elaborates constructor types and produces an `InductiveDecl` without
unresolved metas or holes. Positivity, recursor / computation rule generation,
and exact constructor result checking are kernel / Phase 2 responsibilities.

Indexed families such as `Eq` use leading `forall` in the result type as index
telescope.

```npa
inductive Eq.{u} {A : Sort u} (a : A) : forall (b : A), Prop where
| refl : Eq.{u} a a
```

The MVP supports only core-spec v0.1 simple inductives. Advanced indexed
families, mutual / nested / coinductive definitions, and pattern matching
elaboration are future work.

## 8.5 namespace

```npa
namespace Nat
def double ...
end Nat
```

registers:

```text
Nat.double
```

Namespaces do not appear in core terms.

## 8.6 def/theorem/axiom Elaboration Steps

For `def` and `theorem`:

```text
1. qualify declaration name with the current namespace
2. register universe params `.{...}`
3. confirm declaration name is not duplicate
4. elaborate declaration binders left-to-right without adding the declaration itself to the global scope
5. elaborate the result type after `:` in the local context and confirm it lives in a Sort
6. close the declaration type with Pi binders in source order
7. elaborate body/proof in check mode against the result type
8. close body/proof with Lam binders in source order
9. confirm no metavariable / overload / universe meta remains
10. pass the closed declaration to the Phase 1 kernel
11. register only successful declarations for later items
```

`axiom` performs steps 1-6 and 9-11, with no value/proof.

The MVP forbids self-reference and forward reference. Recursive source
definitions are not supported; use Phase 1 recursor constants explicitly.

Default reducibility: `def` is reducible, `theorem` is opaque. Reducibility
annotations are not in the MVP.

## 8.7 simple inductive Elaboration Steps

Simple inductives need the inductive name available while elaborating
constructors, so Phase 3 uses a temporary global before kernel check.

```text
1. qualify inductive name with the current namespace
2. register universe params
3. confirm planned inductive / constructor / recursor names are not duplicates
4. elaborate declaration binders as params telescope
5. elaborate the type after `:`, reading leading forall telescope as indices and final Sort as sort
6. add temporary global `I` to the env
7. elaborate each constructor type under the params context
8. close constructor types over params
9. build InductiveDecl and pass it to the Phase 1 kernel
10. after kernel success, register I / constructors / recursor for later items
```

The temporary global `I` has the inductive head type as params / indices
telescope. It also carries source interface metadata such as `BinderInfo` during
elaboration. That metadata is not passed to kernel `InductiveDecl.params` /
`indices`.

Examples:

```text
inductive Nat : Type
  params  = []
  indices = []
  sort    = Sort 1

inductive Eq.{u} {A : Sort u} (a : A) : forall (b : A), Prop
  params  = [{A : Sort u}, (a : A)]
  indices = [(b : A)]
  sort    = Sort 0
```

The telescope is read from the WHNF result. Arrow syntax is anonymous `Pi` and
can become an index telescope. If the final result is not `Sort s`, report
`ExpectedSort`.

Constructor source may reference params. Indices are supplied in the return type.

```npa
inductive Eq.{u} {A : Sort u} (a : A) : forall (b : A), Prop where
| refl : Eq.{u} a a
```

The closed constructor type for `refl` is:

```text
Pi A : Sort u, Pi a : A, Eq.{u} A a a
```

Unqualified constructors live in the inductive namespace.

```text
Nat
Nat.zero
Nat.succ
Nat.rec
```

The MVP treats qualified constructor names as relative to the inductive name. It
does not support absolute constructor names outside the inductive namespace.

During constructor type elaboration, references to other constructors or the
recursor from the same block are forbidden. Only imported/local globals, params,
constructor-local binders, and the temporary inductive head are visible.

The recursor is generated or checked by the kernel and added to the Phase 3
global scope / source interface as generated declaration. Phase 2 represents it
with `GlobalRef::LocalGenerated`.

---

# 9. Error Design

Good errors are essential in the surface language.

## 9.1 Parser Error

```text
expected `)` but found `:=`
```

Always include a source span.

## 9.2 Unresolved Name

```text
unknown identifier `addz`
```

Include candidates:

```text
did you mean:
  Nat.add
  Int.add
```

## 9.3 Ambiguous Name

```text
ambiguous name `add`
candidates:
  Nat.add
  Int.add
hint:
  use `Nat.add` or add a type annotation
```

## 9.4 Type Mismatch

```text
type mismatch
  expected: Nat
  actual:   Prop
```

## 9.5 Unsolved Implicit

```text
could not infer implicit argument `A`
in:
  Eq.refl ?A ?x
```

## 9.6 Unsolved Hole

```text
unsolved hole `?proof`

context:
  n : Nat

target:
  n + Nat.zero = n
```

This structured goal is passed to Phase 4 tactics.

## 9.7 Fixed DiagnosticKind Values

API and tests distinguish at least these diagnostic kinds. Hard errors and
warnings are separated by `severity`. Certificate generation fails on any hard
error; warnings never affect trusted payload.

```text
ParserError
ImportResolutionError
ImportAfterItem
NamespaceMismatch
UnknownNamespace
DuplicateDeclaration
DuplicateUniverseParam
InvalidNotation
NotationConflict
UnknownIdentifier
UnknownUniverseParam
AmbiguousName
AmbiguousNotation
TypeMismatch
ExpectedFunctionType
ExpectedSort
BinderInfoMismatch
TooManyArguments
UnsolvedImplicit
UnsolvedUniverseMeta
UnsolvedHole
NamedHoleContextMismatch
OccursCheckFailed
IncompleteDependency
ForwardReference
KernelRejected
ShadowingWarning
DuplicateImportWarning
```

Human messages may change, but `DiagnosticKind`, severity, primary span,
related candidates, and expected/actual types are structured testable data.

---

# 10. Minimum API

## 10.1 parse

```json
{
  "module": "Scratch",
  "verified_imports": [],
  "source": "theorem t (n : Nat) : n = n := Eq.refl n"
}
```

Response:

```json
{
  "status": "ok",
  "surface_ast_id": "ast_123",
  "frontend_state_id": "fe_1"
}
```

`parse` accepts `module` and `verified_imports` because imported notation
metadata and preceding notation declarations affect parsing. A small snippet
parser may exist separately, but the module parse API makes state explicit.

## 10.2 resolve

```json
{
  "surface_ast_id": "ast_123",
  "frontend_state_id": "fe_1"
}
```

Response:

```json
{
  "status": "ok",
  "resolved_names": [
    {
      "surface": "Nat",
      "resolved": "Std.Nat.Nat"
    },
    {
      "surface": "Eq.refl",
      "resolved": "Std.Logic.Eq.refl"
    }
  ]
}
```

## 10.3 elaborate

```json
{
  "module": "Scratch",
  "verified_imports": [],
  "source": "theorem t (n : Nat) : n = n := _"
}
```

Response:

```json
{
  "status": "incomplete",
  "core_type": "Pi Nat (Eq Nat (BVar 0) (BVar 0))",
  "goals": [
    {
      "name": "_",
      "context": [
        {
          "name": "n",
          "type": "Nat"
        }
      ],
      "target": "n = n"
    }
  ]
}
```

## 10.4 elaborate for certificate

```json
{
  "source": "theorem t (n : Nat) : n = n := Eq.refl n",
  "module": "Scratch",
  "verified_imports": [],
  "require_complete": true
}
```

Response:

```json
{
  "status": "ok",
  "core_declaration": {
    "kind": "theorem",
    "name": "t",
    "type_hash": "sha256:...",
    "proof_hash": "sha256:..."
  }
}
```

`module` and `verified_imports` help Phase 3 fix `ElabGlobalRef` and import
hashes. They are not passed directly into the trusted kernel. Each
`verified_imports` entry points to an export interface already checked by the
Phase 2 certificate verifier.

---

# 11. Phase 3 Human Implementation Milestones

Phase 3 Human is implemented in small milestones. Each milestone must return
structured diagnostics, add tests, and keep unresolved holes/metas out of
certificates.

Implementation status (2026-05-21): the MVP milestones in this section are
implemented. Future non-MVP features remain in the exclusion list in section 13.

## 11.1 M1: Parser / Surface AST

- [x] Implement lexer.
- [x] Store `Span` on every token / AST node.
- [x] Parse `import` / `open` / `namespace` / `end`.
- [x] Parse `def` / `theorem` / `axiom` / simple `inductive`.
- [x] Parse `fun` / `forall` / `let` / annotation / application / parenthesized term.
- [x] Desugar `->` / `→` to right-associative anonymous Pi.
- [x] Expand grouped binders `(x y : A)` / `{x y : A}` into `SurfaceBinder` lists.
- [x] Preserve `_` / `?m` as surface holes.
- [x] Preserve `@f` as `ImplicitMode::Explicit`.
- [x] Normalize `Prop` / `Type` / `Type u` to `SurfaceExpr::Sort`.
- [x] Return `ImportAfterItem` when `import` appears after the module prefix.

## 11.2 M2: FrontendState / Name Resolution

- [x] Store current module / namespace stack / open scopes in `FrontendState`.
- [x] Load import interfaces from `verified_imports` and match source imports.
- [x] Handle duplicate imports deterministically.
- [x] Implement lexical scope for namespace / open.
- [x] Separate local context from global declaration table.
- [x] Fix current-module vs imported declaration priority.
- [x] Implement qualified / unqualified name resolution.
- [x] Preserve or reject ambiguous names as `AmbiguousName`.
- [x] Reject forward reference as `ForwardReference`.

## 11.3 M3: Minimal Elaboration / Kernel Handoff

- [x] Implement bidirectional `infer` / `check` skeleton.
- [x] Lower local / global / app / lambda / Pi / let / annotation to core terms.
- [x] Elaborate `def` / `theorem` written with explicit binders.
- [x] Do not add a declaration to the global env during its own elaboration.
- [x] Pass elaborated core declarations to the Phase 1 kernel.
- [x] Return `KernelRejected` when the kernel rejects.
- [x] Add minimal well-typed / ill-typed tests.

## 11.4 M4: Metavariables / Implicit Args / Universe Meta

- [x] Manage term metavariable and universe metavariable stores separately.
- [x] Insert `SyntheticImplicit` metas for implicit binders.
- [x] Do not auto-insert implicit term args in `@` mode.
- [x] Convert `_` and `?m` to `UserHole` metas.
- [x] Compare named-hole context snapshots and return `NamedHoleContextMismatch` on mismatch.
- [x] Store `TypeEq` / `TermEq` / `LevelEq` / `LevelLe` constraints.
- [x] Implement simple unification and occurs check.
- [x] Reject certificate generation when unresolved implicit / universe meta / hole remains.

## 11.5 M5: Notation / Overload Resolution

- [x] Connect notation declarations with namespace / open scope.
- [x] Resolve notation targets to `ElabGlobalRef` when the declaration is processed.
- [x] Reflect prefix / postfix / infix / infixl / infixr in parser binding power.
- [x] Reject notation conflicts as `NotationConflict`.
- [x] Reject non-associative infix chains as `ParserError`.
- [x] Keep overloaded notation candidates in deterministic order.
- [x] Try candidates with transaction / rollback during elaboration.
- [x] Return `AmbiguousNotation` for unresolved notation.

## 11.6 M6: Declaration Coverage / Simple Inductive

- [x] Implement certificate handoff for `def` / `theorem` / `axiom`.
- [x] Confirm axiom use appears in the axiom report.
- [x] Create temporary global for simple inductive.
- [x] Elaborate constructor types in context with the temporary global.
- [x] Convert to core-spec v0.1 `InductiveDecl`.
- [x] Refer to generated declarations such as constructors / recursors with `LocalGenerated`.
- [x] Pass the whole inductive to the kernel and register globals only after success.

## 11.7 M7: Phase 2 Certificate / API / Regression Tests

- [x] Pass fully solved core declarations to the Phase 2 certificate builder.
- [x] Put imported declarations in certificates as refs with `decl_interface_hash`.
- [x] Test deterministic certificate and import hashes.
- [x] Test that axiom reports do not grow unintentionally.
- [x] Stabilize `parse` / `resolve` / `elaborate` APIs.
- [x] Return diagnostic severity and `DiagnosticKind` through APIs.
- [x] Pass `cargo fmt --all`.
- [x] Pass `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] Pass `cargo test --workspace`.

---

# 12. Test Examples

## 12.1 Explicit id

```npa
def id_explicit (A : Type) (x : A) : A :=
  x
```

Check parser, binders, lambda generation, local name resolution, and core
conversion.

## 12.2 implicit id

```npa
def id {A : Type} (x : A) : A :=
  x
```

Check implicit binder and source interface metadata.

## 12.3 implicit argument completion

```npa
theorem refl_nat (n : Nat) : n = n :=
  Eq.refl n
```

Check:

```text
Eq.refl ?A n
?A := Nat
```

## 12.4 notation

```npa
theorem refl_add (n : Nat) : n + Nat.zero = n + Nat.zero :=
  Eq.refl (n + Nat.zero)
```

Check that `+` resolves to `Nat.add` and `=` expands to `Eq`.

## 12.5 hole

```npa
theorem hole_test (n : Nat) : n = n :=
  _
```

Expected:

```text
status: incomplete
goal:
  n : Nat
  ⊢ n = n
```

## 12.6 let

```npa
def let_test (n : Nat) : Nat :=
  let x : Nat := n in x
```

Check core `Let` and ζ-reduction.

## 12.7 ambiguity

```npa
theorem bad (x : _) : x + x = x + x :=
  Eq.refl (x + x)
```

Expected: cannot infer type of `x` or ambiguous notation `+`.

## 12.8 simple inductive

```npa
inductive Nat : Type where
| zero : Nat
| succ : forall (n : Nat), Nat
```

Check constructor type elaboration, `InductiveDecl` generation, and kernel
handoff.

## 12.9 arrow

```npa
def const_nat : Nat -> Nat -> Nat :=
  fun x => fun y => x
```

Check that `->` desugars to right-associative Pi and anonymous binders become
normal core binders.

## 12.10 explicit universe parameter

```npa
def poly_id.{u} {A : Sort u} (x : A) : A :=
  x
```

Check:

```text
universe_params = [u]
undeclared `Sort v` is UnknownUniverseParam
unsolved universe meta at declaration close is UnsolvedUniverseMeta
```

## 12.11 named hole context mismatch

```npa
def bad_named_hole : Nat :=
  let x : Nat := ?m in ?m
```

Expected:

```text
NamedHoleContextMismatch
```

## 12.12 notation conflict

```npa
infixl:65 " + " => Nat.add
infixr:70 " + " => Other.add
```

Expected:

```text
NotationConflict
```

## 12.13 import position

```npa
def x : Nat := Nat.zero
import Std.Nat.Basic
```

Expected:

```text
ImportAfterItem
```

## 12.14 non-associative notation chain

```npa
infix:50 " = " => Eq
theorem bad (a : Nat) (b : Nat) (c : Nat) : Prop :=
  a = b = c
```

Expected:

```text
ParserError
```

## 12.15 grouped binder

```npa
def first (A : Type) (x y : A) : A :=
  x
```

Check that `(x y : A)` expands like `(x : A) (y : A)`, and that annotation `A`
is elaborated outside the scope of `x` / `y`.

---

# 13. Excluded from Phase 3

Deferred to keep the MVP small:

```text
- full typeclass resolution
- coercion search
- macro system
- syntax extensions by users
- multi-token / mixfix notation
- tactic blocks
- pattern matching elaboration
- do notation
- structure projection notation
- term-level numeric literals / overloaded numerals
- aliases
- absolute global name syntax
- source-level recursive definition syntax
- reducibility annotations
- termination checking
- mutual declarations
- automatic universe generalization
- sophisticated universe minimization
```

Typeclass and coercion are powerful but make the elaborator much more complex.
The Phase 3 target is: explicit code works, and simple omissions can be filled
deterministically.

---

# 14. Completion Criteria

Phase 3 is complete when:

```text
- import/open/namespace/end parse
- import is restricted to the module prefix and mid-file import is rejected
- def/theorem/axiom/simple inductive parse
- `->` / `→` desugar to right-associative Pi
- grouped binders `(x y : A)` / `{x y : A}` expand with the specified scope rules
- namespaced names work
- local/global name resolution works
- namespace/open lexical scope works
- notation declarations apply top-to-bottom
- notation conflicts are rejected
- non-associative infix chains are parse errors
- simple infix notation works
- explicit/implicit binders work
- implicit args are inserted as metavariables
- `_` and `?m` become hole goals
- named-hole context mismatch is detected
- bidirectional elaboration works
- simple unification fills the type for Eq.refl n
- universe metavariables are solved or certificate generation rejects them
- unresolved holes reject certificate generation
- solved terms lower to canonical core AST
- simple inductives lower to core-spec v0.1 `InductiveDecl`
- Phase 1 kernel checks the result
- Phase 2 certificate builder can consume the result
```

---

# 15. One-Sentence Summary

Phase 3 is the untrusted layer that converts convenient human syntax into fully
explicit core terms understood by the kernel.

Its core pieces are:

```text
parser:
  text → surface AST

names:
  identifier → local/global reference

notation:
  +, =, etc. → core constant application

implicit args:
  omitted arguments → metavariable → solved by unification

holes:
  incomplete parts → proof goals

simple elaboration:
  surface AST + expected type → fully explicit core AST
```

Phase 3 introduces convenience without trying to be too clever. The target is a
small, deterministic elaborator that reliably produces core terms the kernel can
check.
