# NPA Web Browser MVP Design Todo

Source: `develop/npa-web-browser-mvp-design.md`

## Scope

This task breakdown turns the browser MVP design into implementation-ready
milestones. It covers a local, untrusted Rust web tool for trying Phase 5 Human
proof-state workflows from a browser.

The web tool must stay outside the root NPA workspace hot path. Implement it as
`tools/npa-web`, a nested Cargo workspace with its own `Cargo.lock`. Do not add
it to root `workspace.members`, do not update root `Cargo.lock` for web-only
dependencies, and do not make `./scripts/check-fast.sh` build it.

Non-goals inherited from the design:

- No Node.js runtime, npm dependency, Node package-manager lockfile, frontend
  bundler, React, Vue, Vite, Next, CodeMirror, or Monaco.
- No production multi-user proof service.
- No remote package registry, dependency solver, or network import resolver.
- No browser-side proof checking, WASM kernel, or Web Worker checker in the
  MVP.
- No AI calls, plugin loading, or search service beyond existing local
  `npa-api` library functions.
- No changes to `/machine/*` request grammar, candidate hashes, state
  fingerprints, deterministic budget hashes, kernel behavior, certificate
  format, checker behavior, or proof-corpus tooling.

## Milestones

### W1-01 Create Isolated Web Workspace Skeleton

- Status: Pending
- Depends on: None
- Inputs:
  - `develop/npa-web-browser-mvp-design.md` sections `Proposed Crate`,
    `Workspace Isolation Policy`, and `Definition Of Done For M1`
  - root `Cargo.toml`
  - root `Cargo.lock`
  - `scripts/check-fast.sh`
- Deliverables:
  - `tools/npa-web/Cargo.toml` as a nested workspace root with
    `[workspace] members = ["."]`.
  - `tools/npa-web/Cargo.lock`.
  - Minimal `tools/npa-web/src/main.rs` that starts and exits cleanly or prints
    placeholder startup text.
  - Path dependencies to `../../crates/npa-api`, `../../crates/npa-cert`,
    `../../crates/npa-frontend`, and `../../crates/npa-tactic`.
  - Rust-only web dependencies: `axum`, `tokio`, `go_html_template`,
    `ironframe`, `serde`, and `serde_json`.
- Acceptance criteria:
  - Root `Cargo.toml` is unchanged except for unrelated pre-existing work.
  - Root `Cargo.lock` is unchanged except for unrelated pre-existing work.
  - `tools/npa-web` is not listed in root `workspace.members`.
  - `./scripts/check-fast.sh` does not mention or build `npa-web`.
  - No Node.js/npm files are introduced, including `package.json`,
    `package-lock.json`, `yarn.lock`, or `pnpm-lock.yaml`.
- Verification:
  - `git diff -- Cargo.toml Cargo.lock`
  - `rg -n 'npa-web' Cargo.toml scripts/check-fast.sh`
  - `find tools/npa-web -maxdepth 2 -type f | sort`
  - `cd tools/npa-web && cargo test`
- Notes:
  - This milestone intentionally does not implement the browser UI.
  - The nested workspace keeps web dependencies off the root workspace hot path.

### W1-02 Add Vendored htmx Asset And Static Serving Contract

- Status: Pending
- Depends on: W1-01
- Inputs:
  - `develop/npa-web-browser-mvp-design.md` sections `Proposed Crate`,
    `Routes`, and `Definition Of Done For M1`
  - htmx upstream release artifact and license
- Deliverables:
  - `tools/npa-web/static/vendor/htmx/htmx.min.js`.
  - `tools/npa-web/static/vendor/htmx/LICENSE`.
  - A short version note in `tools/npa-web/static/vendor/htmx/README.md`.
  - Route handler for `GET /assets/htmx.min.js`.
- Acceptance criteria:
  - htmx is served from the repository, not from a CDN.
  - The vendored asset has a recorded upstream version and license.
  - No npm dependency or Node package-manager lockfile is introduced.
  - The route returns JavaScript with an appropriate content type.
- Verification:
  - Review `rg -n 'unpkg|cdn|npm|package-lock|yarn.lock|pnpm-lock' tools/npa-web`
    hits and confirm they do not introduce a runtime CDN fetch, npm dependency,
    or Node package-manager lockfile.
  - `cd tools/npa-web && cargo test`
  - Manual smoke: `curl -i http://127.0.0.1:7420/assets/htmx.min.js`
- Notes:
  - If the server is not yet runnable, keep the route unit-tested and complete
    the manual curl in W1-06.

### W1-03 Implement Template Renderer With go_html_template

- Status: Pending
- Depends on: W1-01
- Inputs:
  - `develop/npa-web-browser-mvp-design.md` sections `Rendering`,
    `Initial UI`, `Routes`, and `Error Handling`
  - `tools/npa-web/templates/`
- Deliverables:
  - `tools/npa-web/src/render.rs`.
  - Server-rendered templates:
    - `page.html`
    - `workspace.html`
    - `goal.html`
    - `messages.html`
    - `verify.html`
  - Renderer tests covering HTML escaping for user source, tactic text, and
    diagnostics.
- Acceptance criteria:
  - Templates are loaded with `include_str!`.
  - `go_html_template` performs context-aware escaping.
  - User-controlled values are not rendered through safe HTML wrappers.
  - Template/rendering failures are converted into short server errors without
    host paths, environment variables, or panic text.
- Verification:
  - `cd tools/npa-web && cargo test render`
  - Review `rg -n 'safe_html|SafeHtml|safe_js|SafeJs' tools/npa-web/src tools/npa-web/templates`
    hits and confirm user-controlled values are not rendered through safe HTML
    or JavaScript wrappers.
  - `git diff --check`
- Notes:
  - Static, repository-owned fragments may use safe wrappers only if a test
    demonstrates why escaping is inappropriate.

### W1-04 Generate And Serve CSS With ironframe

- Status: Pending
- Depends on: W1-03
- Inputs:
  - `develop/npa-web-browser-mvp-design.md` sections `CSS` and `Initial UI`
  - `tools/npa-web/templates/`
- Deliverables:
  - `tools/npa-web/src/style.rs` or equivalent CSS generation module.
  - Route handler for `GET /assets/app.css`.
  - Template class safelist if embedded template scanning misses dynamic
    classes.
  - Tests covering CSS generation for representative classes used by the UI.
- Acceptance criteria:
  - CSS is generated through the `ironframe` Rust library, not Tailwind CLI,
    PostCSS, npm, or Node.js.
  - The generated CSS includes classes used in the first screen and htmx
    partials.
  - The UI uses a restrained operational-tool layout: dense panes, predictable
    controls, small headings, and no decorative marketing-page composition.
  - CSS generation does not touch root `Cargo.lock` or root workspace members.
- Verification:
  - `cd tools/npa-web && cargo test style`
  - Review `rg -n 'tailwind|postcss|node|npm|package-lock|yarn.lock|pnpm-lock' tools/npa-web`
    hits and confirm Tailwind/PostCSS/Node/npm are not build or runtime
    dependencies.
  - Manual smoke after W1-06: `curl -i http://127.0.0.1:7420/assets/app.css`
- Notes:
  - Startup generation is preferred for M1. Build-time generation is deferred
    unless startup cost becomes a measured problem.

### W1-05 Wire Import-Free Human Session Flow

- Status: Pending
- Depends on: W1-01
- Inputs:
  - `develop/npa-web-browser-mvp-design.md` sections `Server Shape`,
    `Initial UI`, `Session Flow`, `Error Handling`, and `Limits And Safety`
  - `crates/npa-api/src/human.rs`
  - `crates/npa-api/src/types.rs`
- Deliverables:
  - `tools/npa-web/src/state.rs` holding `HumanProofSessionStore` behind a
    process-local synchronization primitive.
  - Route/service functions for:
    - session creation from source/module/theorem input
    - proof start
    - tactic run
    - verify
  - Default import-free proof source:
    `theorem id (A : Type) (x : A) : A := by exact x`
  - Request size checks for source and tactic text.
- Acceptance criteria:
  - Session creation calls `create_human_session` and
    `start_human_session_proof`.
  - Tactic execution calls `run_human_tactic`.
  - Verification calls `verify_human_session`.
  - The server does not shell out to `npa` for proof-state operations.
  - Browser input cannot name filesystem paths, execute commands, perform
    network fetches, or add imports in M1.
  - Source input over 128 KiB and tactic input over 4 KiB are rejected with
    user-facing errors.
  - The default proof can be advanced through `intro A`, `intro x`, and
    `exact x` at the service layer.
- Verification:
  - `cd tools/npa-web && cargo test human_flow`
  - `rg -n 'Command::new|std::process|reqwest|ureq|TcpStream|fs::read|File::open' tools/npa-web/src`
  - `git diff --check`
- Notes:
  - M1 uses no verified imports. Standard-library fixture loading belongs to
    W2.

### W1-06 Connect htmx Routes And Browser Smoke

- Status: Pending
- Depends on: W1-02, W1-03, W1-04, W1-05
- Inputs:
  - `develop/npa-web-browser-mvp-design.md` sections `Routes`,
    `Session Flow`, `Initial UI`, and `Definition Of Done For M1`
- Deliverables:
  - `tools/npa-web/src/routes.rs`.
  - `GET /` full-page route.
  - `POST /sessions` workspace partial route.
  - `POST /tactics/run` workspace partial route.
  - `POST /verify` verify-result partial route.
  - Hidden form fields for Human IDs and document version.
  - Startup bind address defaulting to `127.0.0.1:7420`.
- Acceptance criteria:
  - First screen is the usable proof tool, not a landing page.
  - htmx form submissions replace only the intended workspace/verify regions.
  - Server binds to localhost by default and does not bind publicly unless
    explicitly configured.
  - The full browser flow completes the default proof and shows verified status.
  - Route errors are shown as concise UI messages.
- Verification:
  - `cd tools/npa-web && cargo test routes`
  - `cd tools/npa-web && cargo run`
  - Manual browser smoke:
    - open `http://127.0.0.1:7420`
    - create session
    - run `intro A`
    - run `intro x`
    - run `exact x`
    - verify
- Notes:
  - Keep browser verification manual for M1 unless a Rust-only browser test
    harness is introduced later without Node.js/npm.

### W1-07 M1 Regression And Documentation Pass

- Status: Pending
- Depends on: W1-06
- Inputs:
  - `develop/npa-web-browser-mvp-design.md`
  - `develop/npa-web-browser-mvp-design-todo.md`
  - `tools/npa-web/`
  - root `Cargo.toml`
  - root `Cargo.lock`
  - `scripts/check-fast.sh`
- Deliverables:
  - Updated web README or usage section under `tools/npa-web/README.md`.
  - Final M1 verification notes in the implementation PR or commit message.
  - Any narrow design/todo corrections discovered during implementation.
- Acceptance criteria:
  - M1 Definition of Done is satisfied.
  - Root workspace hot path remains unchanged.
  - Root `Cargo.toml`, root `Cargo.lock`, and `scripts/check-fast.sh` do not
    include `npa-web`.
  - No Machine API schema, hash, fingerprint, kernel, certificate, checker, or
    proof-corpus changes are required for M1.
- Verification:
  - `git diff -- Cargo.toml Cargo.lock scripts/check-fast.sh`
  - `rg -n 'npa-web' Cargo.toml scripts/check-fast.sh`
  - `cd tools/npa-web && cargo fmt --all -- --check`
  - `cd tools/npa-web && cargo test`
  - Optional root documentation-only sanity: `git diff --check`
- Notes:
  - Do not run root `./scripts/check-fast.sh` solely because the web tool
    changed; it is intentionally isolated. Run it only if root workspace files
    are changed for another reason.

### W2-01 Add Standard Import Demo

- Status: Pending
- Depends on: W1-07
- Inputs:
  - `develop/npa-web-browser-mvp-design.md` milestone `M2`
  - Existing `npa-api` tests that construct verified `Std.Nat.Basic` and
    `Std.Logic.Eq` Human imports
  - `fixtures/npa-std/`
- Deliverables:
  - Server-side fixture loader for a fixed standard-library demo.
  - Default equality theorem source using `Std.Nat.Basic` and `Std.Logic.Eq`.
  - UI selector for import-free demo vs standard-library demo.
  - Verify panel showing root certificate hash and import/hash summary.
- Acceptance criteria:
  - Fixture loading uses a server-owned fixed path or embedded fixture helper,
    not arbitrary browser-provided filesystem paths.
  - Verified imports are passed explicitly to Human API calls.
  - The standard demo can be completed and verified from the browser.
  - Any helper promoted into `npa-api` is narrowly scoped and does not widen the
    trusted base or Machine hot path.
- Verification:
  - `cd tools/npa-web && cargo test std_demo`
  - Review `rg -n 'PathBuf|File::open|fs::read|canonicalize' tools/npa-web/src`
    hits and confirm they are limited to server-owned fixture allowlists or
    embedded fixtures.
  - Manual browser smoke for the standard demo.
- Notes:
  - If fixture-loading helpers need a design decision, leave the helper local
    to `tools/npa-web` rather than expanding `npa-api`.

### W3-01 Add Package Fixture Browser Mode

- Status: Pending
- Depends on: W2-01
- Inputs:
  - `develop/npa-web-browser-mvp-design.md` milestone `M3`
  - `crates/npa-package`
  - `crates/npa-cli` package command behavior as reference
  - server-side allowlist of package fixture roots
- Deliverables:
  - UI for selecting a package fixture from a fixed allowlist.
  - Server-side package check/build/verify workflow using existing package APIs.
  - Diagnostic display for package command results.
- Acceptance criteria:
  - Browser input cannot provide arbitrary filesystem paths.
  - Package diagnostics remain untrusted metadata and are not presented as proof
    evidence unless backed by certificate/checker verdicts.
  - The mode does not add registry lookup, latest-version resolution, network
    fetches, or dependency solving.
  - Root workspace hot path remains unchanged.
- Verification:
  - `cd tools/npa-web && cargo test package_fixture_mode`
  - Review `rg -n 'registry|latest|network|reqwest|ureq|TcpStream' tools/npa-web/src`
    hits and confirm there is no registry lookup, latest-version resolution, or
    network fetch path.
  - Manual browser smoke with one allowed fixture.
- Notes:
  - Keep the package mode separate from W1 proof-state routes to avoid mixing
    authoring and package-verification semantics.

### W4-01 Add LSP Payload Panels Without JS Tooling

- Status: Pending
- Depends on: W1-07
- Inputs:
  - `develop/npa-web-browser-mvp-design.md` milestone `M4`
  - `crates/npa-api/src/human.rs` `human_lsp_*` payload adapters
  - `develop/phase5-human.md`
- Deliverables:
  - Optional hover/completion/code-action panels rendered server-side.
  - htmx routes for requesting LSP-style payload panels.
  - Documentation of remaining editor limitations.
- Acceptance criteria:
  - A textarea remains the baseline editor.
  - No CodeMirror, Monaco, npm, Node.js, or frontend bundler is introduced.
  - LSP payloads remain Human UI metadata and do not enter `/machine/*`
    responses or certificate payloads.
  - The panels degrade gracefully when no state or selected goal is available.
- Verification:
  - `cd tools/npa-web && cargo test lsp_panels`
  - Review `rg -n 'CodeMirror|Monaco|npm|package.json|MachineTacticCandidate|/machine' tools/npa-web`
    hits and confirm editor package/tooling terms are only negative guards, and
    Machine API terms are only trust-boundary assertions.
  - Manual browser smoke for hover/completion/code-action panel rendering.
- Notes:
  - A real LSP transport server remains out of scope for this milestone.

## Review Findings

The task breakdown has been reviewed against the source design and related Phase
5 documentation. Confirmed findings from the review loop are resolved in this
document:

- W1 was split into smaller milestones so a later implementation agent can
  complete exactly one milestone without guessing.
- The workspace isolation policy is repeated in milestone acceptance criteria to
  prevent `npa-web` from entering the root hot path.
- Node/npm lockfile restrictions are scoped to Node package-manager lockfiles so
  they do not conflict with the required nested `tools/npa-web/Cargo.lock`.
- M1 fixture-free proof authoring is separated from standard-library fixture
  loading and package fixture browsing.
- W2 fixture file-access verification requires review of fixed allowlist or
  embedded-fixture use instead of banning server-owned fixture loading outright.
- Safe-wrapper verification is scoped to user-controlled values so it remains
  consistent with the source design's narrow static-fragment exception.
- Search-based verification commands that mention prohibited tooling now require
  reviewing hits, so README notes and negative tests do not conflict with the
  no-Node/no-CDN/Machine-boundary policies.
- Verification commands run from the nested workspace and do not imply root
  workspace membership.
