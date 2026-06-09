use std::path::PathBuf;

use npa_cli::args::{PackageCommonOptions, PackageGatePlanOptions};
use npa_cli::diagnostic::{CommandExitCode, DiagnosticKind};
use npa_cli::package_gate_plan::run_package_gate_plan;

#[test]
fn package_gate_plan_cli_renders_empty_head_diff_with_trust_boundary() {
    let result = run_package_gate_plan(PackageGatePlanOptions {
        common: PackageCommonOptions {
            root: PathBuf::from("proofs"),
            json: true,
        },
        base: "HEAD".to_owned(),
    });

    assert_eq!(result.exit_code(), CommandExitCode::Success);
    assert_eq!(result.command, "package gate-plan");
    assert_eq!(result.root, "proofs");
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == DiagnosticKind::PackagePolicy
            && diagnostic.reason_code == "gate_plan_impact_class"
            && diagnostic.actual_value.as_deref() == Some("docs-only")
    }));
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.reason_code == "gate_plan_trust_boundary"
            && diagnostic
                .actual_value
                .as_deref()
                .is_some_and(|value| value.contains("never accepts proofs"))
    }));
    let json = result.render_json();
    assert!(json.contains("\"command\":\"package gate-plan\""));
    assert!(json.contains("\"reason_code\":\"gate_plan_required_commands\""));
    assert!(json.contains("git diff --check"));
}

#[test]
fn package_gate_plan_cli_reports_bad_base_without_running_gates() {
    let result = run_package_gate_plan(PackageGatePlanOptions {
        common: PackageCommonOptions {
            root: PathBuf::from("proofs"),
            json: true,
        },
        base: "refs/heads/npa-package-gate-plan-missing-base".to_owned(),
    });

    assert_eq!(result.exit_code(), CommandExitCode::UsageOrInternal);
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == DiagnosticKind::Internal && diagnostic.reason_code == "git_diff_failed"
    }));
    assert!(!result.render_json().contains("check-corpus-full.sh"));
}
