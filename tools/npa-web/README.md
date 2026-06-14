# NPA Web

`npa-web` is a local browser tool for the human-facing NPA proof flow. It is a
nested Rust workspace under `tools/npa-web` so web dependencies and checks stay
off the root NPA hot path.

M1 scope:

- Serve a usable proof page at `GET /`.
- Create an import-free Human session from browser source input.
- Run Human tactics through htmx form posts.
- Verify the closed Human proof state.
- Serve vendored htmx from the repository.
- Generate CSS with the Rust `ironframe` crate.

Out of scope for M1:

- Standard-library fixture loading.
- Package verification workflows.
- Persistence, collaboration, or multi-user isolation.
- JSON API clients.
- Node.js, npm, frontend bundlers, Tailwind CLI, or PostCSS.

## Run

From this directory:

```sh
cargo run
```

The default bind address is:

```text
127.0.0.1:7420
```

Open:

```text
http://127.0.0.1:7420
```

An explicit bind address may be passed for local development:

```sh
cargo run -- --bind 127.0.0.1:9000
```

Do not bind publicly unless that is an intentional local-tool decision for the
current run.

## Default Proof Smoke

The first screen is the proof tool itself. The default source is:

```npa
theorem id (A : Type) (x : A) : A := by exact x
```

Manual browser smoke:

1. Open `http://127.0.0.1:7420`.
2. Click `Create session`.
3. Run `intro A`.
4. Run `intro x`.
5. Run `exact x`.
6. Click `Verify`.
7. Confirm the verify panel shows `verified` and a certificate hash.

## Verification

Use the nested workspace checks:

```sh
cargo fmt --all -- --check
cargo test
cargo clippy --workspace --all-targets -- -D warnings
```

W1-07/M1 regression checks from the repository root:

```sh
git diff -- Cargo.toml Cargo.lock scripts/check-fast.sh
rg -n 'npa-web' Cargo.toml scripts/check-fast.sh
git diff --check
```

The first command should be empty. The `rg` command should have no hits. These
confirm that root `Cargo.toml`, root `Cargo.lock`, and `scripts/check-fast.sh`
do not include `npa-web`.

## Safety Boundary

The browser MVP calls existing Human API functions in process. It does not shell
out to `npa` for proof-state operations.

Browser input is intentionally narrow in M1:

- Source input is limited to 128 KiB.
- Tactic input is limited to 4 KiB.
- Imports are rejected.
- Path-like module/theorem names are rejected.
- Browser input does not name filesystem paths, execute commands, perform
  network fetches, or add imports.

The trusted NPA kernel, certificate format, independent checker, Machine API
schemas, hashes, fingerprints, and proof-corpus tooling are not part of this web
tool milestone.
