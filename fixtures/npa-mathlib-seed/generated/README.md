# Generated Package Artifacts

CLR-09-03 checks in deterministic package artifacts produced by the package
commands:

- `package-lock.json`
- `axiom-report.json`
- `theorem-index.json`
- `publish-plan.json`

These files let a fresh checkout run the base command sequence in check mode.
They remain generated metadata, not trusted proof evidence.
