# NPA Documentation Index

This page is the public routing index for NPA documentation before the
`finitefield-org/npa` repository becomes public.

Public-facing documentation should be English by default. Internal planning,
implementation notes, milestone evidence, and local development notes may
remain Japanese when they are not the first path for public users.

Documentation is not proof evidence. NPA proof acceptance remains based on
canonical `.npcert` bytes, Rust kernel / verifier verdicts, source-free checker
verdicts, and deterministic proof artifact hashes.

## Classification Table

| Area | Audience | Language target | Status | Primary paths |
| --- | --- | --- | --- | --- |
| Public repository entry | Users, package authors, auditors | English | `README.md` cleaned in PUB-01; `LICENSE` added in PUB-02; `CONTRIBUTING.md` added in PUB-03 | `README.md`, `LICENSE`, `CONTRIBUTING.md` |
| Public docs router | Users, package authors, auditors | English | Expanded documentation index in PUB-04; `doc/index.md` alias added in PUB-10 | `doc/README.md`, `doc/index.md` |
| Toolchain reference | External theorem package maintainers | English | Current ref is `v0.1.1`; `v0.1.0` is historical and reviewed in PUB-06 | `doc/npa-toolchain-reference-v0.1.1.md`, `doc/npa-toolchain-reference-v0.1.0.md` |
| External package CI | External theorem package maintainers | English | CI template guide cleaned in PUB-05; external CI guide cleaned in PUB-07 | `doc/external-theorem-library-ci.md`, `ci-templates/github-actions/README.md` |
| External checker docs | High-trust checker integrators | English | Cleaned in PUB-08; optional high-trust path only | `checkers/npa-checker-ext/README.md` |
| Public examples and fixtures | Users only when explicitly linked from public docs | English for linked examples; otherwise internal label | Visibility decided in PUB-09 | `fixtures/*/README.md`, `proofs/README.md` |
| Design specifications | Implementers and auditors | English preferred, mixed internal detail accepted until cleanup | Internal/reference docs | `doc/core-spec-v0.1.md`, `doc/overall-design.md`, `doc/phase*.md` |
| Internal milestones and evidence | Maintainers and development agents | Japanese or mixed English/Japanese allowed | Internal | `doc/*-todo.md`, `doc/npa-standalone-repo-activation.md`, `doc/registry-readiness.md` |

## User And Package Author Docs

These docs are intended to be safe entry points for public readers once the
corresponding cleanup milestones are complete:

- [Repository README](../README.md): overview, trust boundary, build from
  source, package verification quick start, and repository layout.
- [Contributing](../CONTRIBUTING.md): contribution policy, local gates, corpus
  gate triggers, `unsafe` Rust policy, and working-tree etiquette.
- [MIT License](../LICENSE): repository license.
- [External Theorem Library CI](external-theorem-library-ci.md): package CI
  guide for external theorem libraries. Cleaned in PUB-07.
- [GitHub Actions CI Templates](../ci-templates/github-actions/README.md):
  copyable package workflow guidance. Cleaned in PUB-05.

## Toolchain Reference Docs

- [Toolchain Reference v0.1.1](npa-toolchain-reference-v0.1.1.md): current
  SRA-02-compatible `npa` toolchain ref for external theorem packages.
- [Historical Toolchain Reference v0.1.0](npa-toolchain-reference-v0.1.0.md):
  SRA-01 ref retained for audit context. Do not recommend it as the current
  external package pin.

## External Checker Docs

- [npa-checker-ext README](../checkers/npa-checker-ext/README.md): optional
  high-trust external checker guidance. Cleaned in PUB-08. Base package CI
  remains reference-checker-only.

## Public Examples And Fixtures

These fixture README files are safe public examples:

- [npa-std fixture](../fixtures/npa-std/README.md): standalone standard-library
  package verification example.
- [npa-mathlib fixture](../fixtures/npa-mathlib/README.md): public
  `Mathlib.*` theorem-library package example.
- [npa-mathlib downstream fixture](../fixtures/npa-mathlib-downstream/README.md):
  downstream certificate-vendoring example without a registry server.

These fixture and corpus notes are internal/test documentation, not public
package-author entry points:

- [npa-mathlib-seed fixture](../fixtures/npa-mathlib-seed/README.md)
- [npa-mathlib-seed contributing note](../fixtures/npa-mathlib-seed/CONTRIBUTING.md)
- [npa-mathlib-seed generated artifact note](../fixtures/npa-mathlib-seed/generated/README.md)
- [npa-mathlib-seed downstream fixture](../fixtures/npa-mathlib-seed-downstream/README.md)
- [Vendored seed artifact note](../fixtures/npa-mathlib-seed-downstream/vendor/npa-mathlib-seed/README.md)
- [Proof corpus README](../proofs/README.md)

## Design And Specification Docs

These docs describe the system design and implementation phases. Some are
mixed-language internal references until later cleanup:

- [Core Specification v0.1](core-spec-v0.1.md)
- [Overall Design](overall-design.md)
- [Phase 0: Core Spec](phase0.md)
- [Phase 1: Kernel](phase1.md)
- [Phase 2: Certificate](phase2.md)
- [Phase 3 Human Surface](phase3-human.md)
- [Phase 3 AI Machine Surface](phase3-ai.md)
- [Phase 4 Human Tactic](phase4-human.md)
- [Phase 4 AI Machine Tactics](phase4-ai.md)
- [Phase 5 Human IDE/API](phase5-human.md)
- [Phase 5 AI Machine IDE/API](phase5-ai.md)
- [Phase 6 Human Library](phase6-human.md)
- [Phase 6 AI Machine Standard Library](phase6-ai.md)
- [Phase 7 AI Search](phase7-ai.md)
- [Phase 8 Human Independent Checker](phase8-human.md)
- [Phase 8 AI Checker Audit Automation](phase8-ai.md)
- [Phase 9 Human Advanced Features](phase9-human.md)
- [Phase 9 AI Advanced Automation](phase9-ai.md)

## Release And Evidence Docs

These docs record activation, release-readiness, and package ecosystem
evidence. They are not proof evidence:

- [Public Repository Readiness Todo](public-repository-readiness-todo.md)
- [Standalone Repository Activation](npa-standalone-repo-activation.md)
- [Registry Readiness](registry-readiness.md)
- [NPA Mathlib Public Release Plan](npa-mathlib-public-release-plan.md)
- [Community Library Roadmap](community-library-roadmap.md)

## Internal And Development Docs

These docs may remain Japanese or mixed-language because they are planning,
implementation, or evidence records rather than the public user entry path:

- `phase*-todo.md`
- `community-library-roadmap-*-todo.md`
- `npa-checker-ext-ocaml*.md`
- [Internal README Notes](internal-readme-notes-ja.md)
- [Proof Corpus AI Workflow](proof-corpus-ai-workflow.md), unless it is later
  promoted to public user guidance.
- `proofs/**` planning notes, unless a specific proof package guide is later
  promoted to public user guidance.
- `fixtures/npa-mathlib-seed*/**` README and CONTRIBUTING files, because
  PUB-09 marks them as internal/test fixture notes.

## Current Public Readiness Notes

- The current public toolchain recommendation is `NPA_GIT_TAG=v0.1.1`.
- `v0.1.0` is historical and should not be recommended as the current external
  package toolchain ref.
- Public docs must not imply that GitHub metadata, CI status, release pages,
  theorem indexes, publish plans, source files, replay files, tactic traces, or
  AI traces are trusted proof evidence.
- `finitefield-org/npa` still needs the remaining PUB milestones before public
  visibility is the default repository posture.
