use std::path::PathBuf;
use std::process::Command;

use npa_cli::args::{
    parse_cli_args, CliAction, CliCommand, HelpTopic, PackageChecker, PackageCommand, UsageReason,
};

fn parse(args: &[&str]) -> CliAction {
    parse_cli_args(args.iter().copied()).unwrap()
}

fn parse_error(args: &[&str]) -> npa_cli::args::CliUsageError {
    parse_cli_args(args.iter().copied()).unwrap_err()
}

#[test]
fn package_cli_args_parses_check_defaults_root_to_current_directory() {
    let action = parse(&["package", "check"]);

    let CliAction::Run(CliCommand::Package(PackageCommand::Check(options))) = action else {
        panic!("expected package check command");
    };
    assert_eq!(options.root, PathBuf::from("."));
    assert!(!options.json);
}

#[test]
fn package_cli_args_parses_common_root_and_json_flags() {
    let action = parse(&["package", "check-hashes", "--root", "proofs", "--json"]);

    let CliAction::Run(CliCommand::Package(PackageCommand::CheckHashes(options))) = action else {
        panic!("expected package check-hashes command");
    };
    assert_eq!(options.root, PathBuf::from("proofs"));
    assert!(options.json);
}

#[test]
fn package_cli_args_parses_build_certs_check_mode() {
    let action = parse(&[
        "package",
        "build-certs",
        "--root=proofs",
        "--check",
        "--json",
    ]);

    let CliAction::Run(CliCommand::Package(PackageCommand::BuildCerts(options))) = action else {
        panic!("expected package build-certs command");
    };
    assert_eq!(options.common.root, PathBuf::from("proofs"));
    assert!(options.common.json);
    assert!(options.check);
}

#[test]
fn package_cli_args_parses_axiom_report_check_mode() {
    let action = parse(&[
        "package",
        "axiom-report",
        "--root=proofs",
        "--check",
        "--json",
    ]);

    let CliAction::Run(CliCommand::Package(PackageCommand::AxiomReport(options))) = action else {
        panic!("expected package axiom-report command");
    };
    assert_eq!(options.common.root, PathBuf::from("proofs"));
    assert!(options.common.json);
    assert!(options.check);
}

#[test]
fn package_cli_args_defaults_verify_certs_checker_to_reference() {
    let action = parse(&["package", "verify-certs"]);

    let CliAction::Run(CliCommand::Package(PackageCommand::VerifyCerts(options))) = action else {
        panic!("expected package verify-certs command");
    };
    assert_eq!(options.checker, PackageChecker::Reference);
    assert_eq!(options.common.root, PathBuf::from("."));
}

#[test]
fn package_cli_args_parses_verify_certs_fast_checker() {
    let action = parse(&[
        "package",
        "verify-certs",
        "--checker",
        "fast",
        "--root",
        "proofs",
    ]);

    let CliAction::Run(CliCommand::Package(PackageCommand::VerifyCerts(options))) = action else {
        panic!("expected package verify-certs command");
    };
    assert_eq!(options.checker, PackageChecker::Fast);
    assert_eq!(options.common.root, PathBuf::from("proofs"));
}

#[test]
fn package_cli_args_rejects_external_checker_as_unsupported() {
    let error = parse_error(&["package", "verify-certs", "--checker", "external"]);

    assert_eq!(error.reason, UsageReason::UnsupportedChecker);
    assert_eq!(error.flag.as_deref(), Some("--checker"));
    assert_eq!(error.value.as_deref(), Some("external"));
}

#[test]
fn package_cli_args_rejects_unsupported_clr04_flags() {
    for flag in [
        "--changed",
        "--changed=true",
        "--all",
        "--all=true",
        "--registry",
        "--registry=local",
        "--network",
        "--network=on",
        "--include-source",
        "--include-source=true",
        "--include-replay",
        "--include-replay=true",
        "--include-ai-traces",
        "--include-ai-traces=true",
    ] {
        let error = parse_error(&["package", "check", flag]);
        assert_eq!(error.reason, UsageReason::UnsupportedFlag, "{flag}");
        assert_eq!(error.flag.as_deref(), Some(flag), "{flag}");
    }

    let checker_error = parse_error(&["package", "axiom-report", "--checker=external"]);
    assert_eq!(checker_error.reason, UsageReason::UnsupportedFlag);
    assert_eq!(checker_error.flag.as_deref(), Some("--checker=external"));
}

#[test]
fn package_cli_args_rejects_unknown_commands_and_flags() {
    let command_error = parse_error(&["package", "publish-plan"]);
    assert_eq!(command_error.reason, UsageReason::UnknownCommand);
    assert_eq!(
        command_error.command.as_deref(),
        Some("package publish-plan")
    );

    let flag_error = parse_error(&["package", "check", "--mystery"]);
    assert_eq!(flag_error.reason, UsageReason::UnknownFlag);
    assert_eq!(flag_error.flag.as_deref(), Some("--mystery"));
}

#[test]
fn package_cli_args_rejects_duplicate_flags() {
    let root_error = parse_error(&["package", "check", "--root", "proofs", "--root", "other"]);
    assert_eq!(root_error.reason, UsageReason::DuplicateFlag);
    assert_eq!(root_error.flag.as_deref(), Some("--root"));

    let json_error = parse_error(&["package", "check", "--json", "--json"]);
    assert_eq!(json_error.reason, UsageReason::DuplicateFlag);
    assert_eq!(json_error.flag.as_deref(), Some("--json"));

    let checker_error = parse_error(&[
        "package",
        "verify-certs",
        "--checker",
        "fast",
        "--checker",
        "reference",
    ]);
    assert_eq!(checker_error.reason, UsageReason::DuplicateFlag);
    assert_eq!(checker_error.flag.as_deref(), Some("--checker"));

    let build_error = parse_error(&["package", "build-certs", "--check", "--check"]);
    assert_eq!(build_error.reason, UsageReason::DuplicateFlag);
    assert_eq!(build_error.flag.as_deref(), Some("--check"));

    let axiom_report_error = parse_error(&["package", "axiom-report", "--check", "--check"]);
    assert_eq!(axiom_report_error.reason, UsageReason::DuplicateFlag);
    assert_eq!(axiom_report_error.flag.as_deref(), Some("--check"));
}

#[test]
fn package_cli_args_rejects_missing_flag_values() {
    let root_error = parse_error(&["package", "check", "--root"]);
    assert_eq!(root_error.reason, UsageReason::MissingFlagValue);
    assert_eq!(root_error.flag.as_deref(), Some("--root"));

    let checker_error = parse_error(&["package", "verify-certs", "--checker"]);
    assert_eq!(checker_error.reason, UsageReason::MissingFlagValue);
    assert_eq!(checker_error.flag.as_deref(), Some("--checker"));

    let checker_equals_error = parse_error(&["package", "verify-certs", "--checker="]);
    assert_eq!(checker_equals_error.reason, UsageReason::MissingFlagValue);
    assert_eq!(checker_equals_error.flag.as_deref(), Some("--checker"));
}

#[test]
fn package_cli_args_parses_help_topics() {
    assert_eq!(parse(&["--help"]), CliAction::Help(HelpTopic::Root));
    assert_eq!(
        parse(&["package", "--help"]),
        CliAction::Help(HelpTopic::Package)
    );
    assert_eq!(
        parse(&["package", "check", "--help"]),
        CliAction::Help(HelpTopic::PackageCheck)
    );
    assert_eq!(
        parse(&["package", "verify-certs", "--help"]),
        CliAction::Help(HelpTopic::PackageVerifyCerts)
    );
    assert_eq!(
        parse(&["package", "axiom-report", "--help"]),
        CliAction::Help(HelpTopic::PackageAxiomReport)
    );
}

#[test]
fn package_cli_args_binary_reports_deterministic_usage_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_npa"))
        .args(["package", "verify-certs", "--checker", "external"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "package verify-certs: failed\nerror Usage unsupported_checker field=--checker actual=external\n"
    );
}

#[test]
fn package_cli_args_binary_reports_json_usage_error_when_requested() {
    let output = Command::new(env!("CARGO_BIN_EXE_npa"))
        .args(["package", "check", "--mystery", "--json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"schema\":\"npa.package.command_result.v0.1\""));
    assert!(stdout.contains("\"kind\":\"Usage\""));
    assert!(stdout.contains("\"reason_code\":\"unknown_flag\""));
    assert!(stdout.contains("\"field\":\"--mystery\""));
}
