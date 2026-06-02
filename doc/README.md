# NPA Documentation Map

This page classifies NPA documentation before the `finitefield-org/npa`
repository becomes public.

Public-facing documentation should be English by default. Internal planning,
implementation notes, milestone evidence, and local development notes may
remain Japanese when they are not the first path for public users.

Documentation is not proof evidence. NPA proof acceptance remains based on
canonical `.npcert` bytes, Rust kernel / verifier verdicts, source-free checker
verdicts, and deterministic proof artifact hashes.

## Classification Table

| Area | Audience | Language target | Status | Primary paths |
| --- | --- | --- | --- | --- |
| Public repository entry | Users, package authors, auditors | English | `README.md` cleaned in PUB-01; `LICENSE` added in PUB-02; `CONTRIBUTING.md` pending PUB-03 | `README.md`, `LICENSE`, `CONTRIBUTING.md` |
| Public docs router | Users, package authors, auditors | English | Active classification page | `doc/README.md` |
| Toolchain reference | External theorem package maintainers | English | Current ref is `v0.1.1`; `v0.1.0` is historical | `doc/npa-toolchain-reference-v0.1.1.md`, `doc/npa-toolchain-reference-v0.1.0.md` |
| External package CI | External theorem package maintainers | English | Pending public-doc cleanup in PUB-05/PUB-07 | `doc/external-theorem-library-ci.md`, `ci-templates/github-actions/README.md` |
| External checker docs | High-trust checker integrators | English if included in first public user path | Pending decision in PUB-08 | `checkers/npa-checker-ext/README.md` |
| Public examples and fixtures | Users only when explicitly linked from public docs | English for linked examples; otherwise internal label | Pending decision in PUB-09 | `fixtures/*/README.md`, `proofs/README.md` |
| Design specifications | Implementers and auditors | English preferred, mixed internal detail accepted until cleanup | Internal/reference docs | `doc/core-spec-v0.1.md`, `doc/overall-design.md`, `doc/phase*.md` |
| Internal milestones and evidence | Maintainers and development agents | Japanese or mixed English/Japanese allowed | Internal | `doc/*-todo.md`, `doc/npa-standalone-repo-activation.md`, `doc/registry-readiness.md` |

## Public-Facing Docs

These docs are intended to be safe entry points for public readers once the
corresponding cleanup milestones are complete:

- `README.md`: repository overview and quick start. Cleaned in PUB-01.
- `LICENSE`: repository MIT license. Added in PUB-02.
- `CONTRIBUTING.md`: contributor policy and local gates. Creation target:
  PUB-03.
- `doc/README.md`: this documentation classification page.
- `doc/npa-toolchain-reference-v0.1.1.md`: current SRA-02-compatible toolchain
  reference for external theorem packages.
- `doc/external-theorem-library-ci.md`: package CI contract for external
  theorem libraries. Cleanup target: PUB-07.
- `ci-templates/github-actions/README.md`: copyable GitHub Actions template
  guidance. Cleanup target: PUB-05.
- `checkers/npa-checker-ext/README.md`: external checker guidance if external
  checker users are included in the first public release path. Decision target:
  PUB-08.

## Internal And Development Docs

These docs may remain Japanese or mixed-language because they are planning,
implementation, or evidence records rather than the public user entry path:

- `doc/phase*.md`
- `doc/*-todo.md`
- `doc/community-library-roadmap*.md`
- `doc/npa-standalone-repo-activation.md`
- `doc/registry-readiness.md`
- `doc/internal-readme-notes-ja.md`
- `doc/proof-corpus-ai-workflow.md`, unless it is later promoted to public
  user guidance.
- `proofs/**` planning notes, unless a specific proof package guide is later
  promoted to public user guidance.
- `fixtures/**` README and CONTRIBUTING files, unless PUB-09 marks a fixture
  as a public example.

## Current Public Readiness Notes

- The current public toolchain recommendation is `NPA_GIT_TAG=v0.1.1`.
- `v0.1.0` is historical and should not be recommended as the current external
  package toolchain ref.
- Public docs must not imply that GitHub metadata, CI status, release pages,
  theorem indexes, publish plans, source files, replay files, tactic traces, or
  AI traces are trusted proof evidence.
- `finitefield-org/npa` still needs the remaining PUB milestones before public
  visibility is the default repository posture.
