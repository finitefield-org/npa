# GitHub Actions Template Location

This directory is reserved for copyable external theorem library CI templates.
It is intentionally outside `.github/workflows` so these files are not active
local workflows for the `npa` repository.

Planned template files:

```text
npa-package-pr.yml
npa-package-release.yml
```

CLR-07-01 defines the contract and directory location before adding concrete
YAML. The contract source is:

```text
doc/external-theorem-library-ci.md
```

External theorem libraries should copy templates from this directory once later
CLR-07 milestones add them. They should not copy local `npa` repository gates
such as:

```sh
scripts/phase8-release-audit.sh
scripts/phase9-regression.sh
```

Those scripts are repository development gates, not external theorem library CI.

The base contract uses full-package package commands:

```sh
npa package check --root . --json
npa package build-certs --root . --check --json
npa package check-hashes --root . --json
npa package verify-certs --root . --checker reference --json
npa package axiom-report --root . --check --json
npa package index --root . --check --json
```

Release templates may add the fast verifier as a labeled non-reference gate:

```sh
npa package verify-certs --root . --checker fast --json
```

Base CLR-07 templates must not use `--changed`, `--all`, `--checker external`,
`--registry`, `--network`, or `--latest`. They must not contact an NPA package
registry, use hidden package caches, or resolve imports by implicit latest
version.

CI output is not proof evidence. The proof boundary remains canonical
certificate artifacts plus source-free checker or kernel verifier verdicts.
