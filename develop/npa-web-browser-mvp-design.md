# NPA Web Browser MVP Design

Status: design draft.

This document defines the first browser-accessible human tool for NPA. It is a
thin, untrusted web surface over the existing Phase 5 Human IDE/API library
layer. The goal is to let a developer open a local browser, create a Human proof
session, inspect goals, run tactics, and verify the resulting certificate
without introducing Node.js, npm, or any new trusted proof boundary.

## Goals

- Provide a local browser UI for trying the Human proof-state workflow.
- Keep the server in Rust so it can call `npa-api`, `npa-frontend`, and
  `npa-tactic` directly.
- Use htmx for browser interaction.
- Use `go_html_template` for server-rendered HTML templates.
- Use `ironframe` for Tailwind CSS-compatible styling without Node.js or npm.
- Keep the UI, HTTP routes, templates, CSS generation, and htmx behavior fully
  outside the trusted base.

## Non-Goals

- No production multi-user proof service.
- No remote package registry, dependency solver, or network import resolver.
- No browser-side proof checking, WASM kernel, or Web Worker checker in the MVP.
- No CodeMirror, Monaco, React, Vue, Vite, Next, npm, or Node.js build pipeline.
- No AI calls, plugin loading, or search service beyond existing local
  `npa-api` library functions.
- No changes to `/machine/*` request grammar, candidate hashes, state
  fingerprints, or deterministic budget hashes.

## Trust Boundary

The web tool is untrusted convenience infrastructure.

```text
Untrusted:
  browser UI
  htmx requests
  Rust HTTP server
  go_html_template templates
  ironframe-generated CSS
  Human source text
  Human tactic text
  display strings
  session IDs in forms

Trusted:
  canonical certificate bytes
  Rust kernel / verifier verdict
  source-free checker verdict
  deterministic certificate / import hashes
```

The server may report that a tactic step succeeded, but proof acceptance still
requires the final certificate handoff and kernel / checker verdict. The kernel
must not depend on HTTP, template rendering, htmx, CSS generation, filesystem
serving, or browser state.

## Proposed Crate

Add a new binary crate as a separate workspace:

```text
tools/npa-web/
  Cargo.toml
  Cargo.lock
  src/main.rs
  src/routes.rs
  src/render.rs
  src/state.rs
  templates/
    page.html
    workspace.html
    goal.html
    messages.html
    verify.html
  static/
    vendor/htmx/htmx.min.js
    vendor/htmx/LICENSE
```

`tools/npa-web` is deliberately not a member of the root NPA workspace. Its
`Cargo.toml` should be a nested workspace root so running Cargo inside the web
tool does not require adding it to the parent `Cargo.toml`.

```toml
[workspace]
members = ["."]
resolver = "2"

[package]
name = "npa-web"
version = "0.1.0"
edition = "2021"
license = "Apache-2.0"
repository = "https://github.com/finitefield-org/npa"
```

The web crate depends on NPA crates through explicit path dependencies:

```toml
npa-api = { path = "../../crates/npa-api" }
npa-cert = { path = "../../crates/npa-cert" }
npa-frontend = { path = "../../crates/npa-frontend" }
npa-tactic = { path = "../../crates/npa-tactic" }
axum = "0.8"
tokio = { version = "1", features = ["macros", "rt-multi-thread", "net"] }
go_html_template = "0.2"
ironframe = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

The htmx asset is vendored as a static file from an explicit upstream release.
It must not be fetched from `unpkg`, npm, or a runtime CDN. Record its version
and license in the vendored directory.

## Workspace Isolation Policy

The web tool must stay out of the NPA implementation hot path.

Rules:

- Do not add `tools/npa-web` to the root `workspace.members`.
- Do not update the root `Cargo.lock` merely to add web-only dependencies.
- Keep the web tool's dependency lockfile under `tools/npa-web/Cargo.lock`.
- Do not make `./scripts/check-fast.sh` build `npa-web`.
- Do not make root-level `cargo clippy --workspace` or
  `cargo test --workspace` include `npa-web`.
- Run web checks explicitly from the nested workspace only.

This means normal kernel, certificate, checker, package, frontend, tactic,
API, CLI, and proof-corpus work keeps the same root workspace hot path. The web
tool may depend on NPA crates, but NPA crates must not depend on the web tool.

## Server Shape

Use `axum` with a local default bind address:

```text
127.0.0.1:7420
```

The server is local development tooling. It should not bind publicly unless a
caller explicitly passes a host / port option.

The initial server state is single-process and in-memory:

```rust
struct WebState {
    human_store: Mutex<HumanProofSessionStore>,
}
```

This keeps the MVP narrow. Session persistence, collaboration, and multi-user
isolation are deferred.

## Rendering

All HTML is rendered server-side with `go_html_template`.

Rules:

- Use `include_str!` for templates.
- Parse templates at startup or through a small renderer cache.
- Let `go_html_template` perform context-aware escaping.
- Do not mark user source, tactic text, diagnostics, or theorem display as safe
  HTML.
- Only static, repository-owned fragments may use safe HTML wrappers, and the
  MVP should avoid them unless necessary.

HTML partials are first-class route responses. htmx swaps these partials into
the page after form submissions.

## CSS

Use `ironframe` as the Rust-only Tailwind-compatible CSS engine.

Design options, in preferred order:

1. In-process generation at server startup.
   - Scan the embedded template strings and any explicit class safelist.
   - Generate CSS through `ironframe::scanner` and `ironframe::generator`.
   - Serve the generated stylesheet from `/assets/app.css`.
2. Build-time generation through a Rust build step.
   - Use an `xtask` or `build.rs` only if startup generation becomes too slow.
   - The build step must still be Rust-only.
3. CLI generation with the `ironframe` binary.
   - Acceptable for manual release artifact generation.
   - Not the preferred server path because the web crate already depends on the
     Rust library.

No Tailwind CLI, PostCSS, npm package, or Node.js process is allowed.

The MVP should use a restrained operational-tool style: dense layout, clear
panes, small headings, and predictable controls. Avoid marketing-page structure,
decorative backgrounds, and one-hue themes.

## Initial UI

The first screen is the usable tool, not a landing page.

Layout:

```text
+-------------------------------------------------------------+
| header: NPA Web                                             |
+-----------------------------+-------------------------------+
| source editor textarea      | proof state                   |
| module / theorem controls   | selected goal                 |
| create session button       | context / target              |
|                             | tactic input + run button     |
| messages                    | verify result                 |
+-----------------------------+-------------------------------+
```

Initial default source:

```npa
theorem id (A : Type) (x : A) : A := by exact x
```

Default module:

```text
Scratch
```

Default theorem:

```text
Scratch.id
```

This import-free example allows the MVP to exercise the Human session,
proof-state, tactic, and verify paths without first solving package fixture
loading in the web server.

Expected interactive tactic sequence:

```text
intro A
intro x
exact x
```

## Routes

HTML routes:

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/` | Render full page with default source and empty workspace. |
| `POST` | `/sessions` | Create a Human session and start a theorem proof. Returns workspace partial. |
| `POST` | `/tactics/run` | Run one Human tactic against the selected goal. Returns workspace partial. |
| `POST` | `/verify` | Verify the current closed state. Returns verify-result partial. |
| `GET` | `/assets/app.css` | Return `ironframe` generated CSS. |
| `GET` | `/assets/htmx.min.js` | Return vendored htmx. |

Form payloads use ordinary `application/x-www-form-urlencoded` htmx requests.
JSON endpoints are deferred until there is a separate API client need.

## Session Flow

```text
GET /
  -> full page

POST /sessions
  source, module, theorem
  -> create_human_session
  -> start_human_session_proof
  -> get_human_state_by_id
  -> workspace partial

POST /tactics/run
  session_id, document_id, document_version, state_id, goal_id, tactic
  -> run_human_tactic
  -> get_human_state_by_id(new_state_id or old_state_id)
  -> workspace partial

POST /verify
  session_id, document_id, document_version, state_id
  -> verify_human_session
  -> verify-result partial
```

Hidden form fields may carry Human IDs using their `as_str()` / `wire()` values.
The server reconstructs them with `new_unchecked()` because the IDs were
allocated by this same local process. A later public JSON API should validate
wire grammar more explicitly.

## Error Handling

The UI should show structured failures without panicking the server.

Initial display fields:

- session creation diagnostics: `HumanDiagnostic.message`
- tactic status: `HumanTacticRunStatus::as_str()`
- tactic errors: `HumanTacticRunErrorKind::as_str()` and report message
- verify errors: open goal IDs or certificate handoff message
- template/rendering errors: HTTP 500 with a short server-side error message

Do not expose host paths, environment variables, or raw panic text in browser
responses.

## Limits And Safety

MVP request limits:

- Maximum source length: 128 KiB.
- Maximum tactic length: 4 KiB.
- Only local in-memory imports: none for M1.
- No filesystem path input from the browser.
- No command execution from browser input.
- No network fetch from browser input.

Server lifecycle:

- Start explicitly from the nested workspace with `cd tools/npa-web && cargo run`.
- Print the local URL on startup.
- Shutdown with Ctrl-C.

## Milestones

### M1: Import-Free Human Proof UI

Deliver:

- `tools/npa-web` binary.
- Full page rendered by `go_html_template`.
- htmx session creation, tactic run, and verify partial swaps.
- `ironframe` generated CSS served from Rust.
- Vendored htmx static asset.
- Default `Scratch.id` proof workflow.

Verification:

```sh
cd tools/npa-web
cargo fmt --all -- --check
cargo test
cargo run
```

Manual browser smoke:

```text
open http://127.0.0.1:7420
create session
run: intro A
run: intro x
run: exact x
verify
```

### M2: Standard Import Demo

Deliver:

- Load verified `Std.Nat.Basic` and `Std.Logic.Eq` fixture modules in-process.
- Provide default equality theorem source.
- Show goal context, target, tactic result, and verify hashes.

This milestone may reuse helper logic from existing `npa-api` tests, but test
helpers should be promoted into public library functions only if that reduces
duplication without widening the trusted base.

### M3: Package Fixture Browser Mode

Deliver:

- Select a local package fixture root from a fixed server-side allowlist.
- Build / verify certificates through existing package APIs.
- Display package command results as untrusted diagnostics.

Browser input must not become arbitrary filesystem access.

### M4: Editor And LSP Payloads

Deliver:

- Keep `<textarea>` as the baseline editor.
- Add optional hover/completion/code-action panels based on existing
  `human_lsp_*` payload adapters.
- Continue to avoid npm and Node.js.

## Open Questions

- Whether `ironframe` CSS should be generated at startup or committed as a
  generated artifact for release builds.
- Whether `go_html_template` templates should be parsed once at startup or per
  request during early development.
- Whether session IDs need a browser-visible CSRF token for local-only MVP.
- Whether M2 fixture-loading helpers belong in `npa-api` or only in `npa-web`.
- How much certificate detail the verify panel should show by default.

## Definition Of Done For M1

- No Node.js runtime, npm dependency, Node package-manager lockfile such as
  `package-lock.json`, `yarn.lock`, or `pnpm-lock.yaml`, or frontend bundler is
  introduced.
- The only JavaScript dependency is vendored htmx.
- HTML is rendered through `go_html_template`.
- CSS is produced through `ironframe`.
- The server calls existing Human API functions rather than shelling out to
  `npa` for proof-state operations.
- The default proof can be completed and verified from a browser.
- Root `Cargo.toml`, root `Cargo.lock`, root `./scripts/check-fast.sh`, and
  root workspace CI / local gates do not include `npa-web`.
- Kernel, certificate, checker, Machine API hashes, and proof-corpus tooling are
  unchanged.
