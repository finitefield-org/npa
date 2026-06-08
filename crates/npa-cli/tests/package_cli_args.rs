use std::path::PathBuf;
use std::process::Command;

use npa_cli::args::{
    parse_cli_args, CliAction, CliCommand, HelpTopic, PackageAuditCacheMode, PackageChecker,
    PackageCommand, UsageReason,
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
fn package_cli_args_parses_package_index_check_mode() {
    let action = parse(&["package", "index", "--root=proofs", "--check", "--json"]);

    let CliAction::Run(CliCommand::Package(PackageCommand::Index(options))) = action else {
        panic!("expected package index command");
    };
    assert_eq!(options.common.root, PathBuf::from("proofs"));
    assert!(options.common.json);
    assert!(options.check);
}

#[test]
fn package_cli_args_parses_publish_plan_check_mode() {
    let action = parse(&[
        "package",
        "publish-plan",
        "--root=proofs",
        "--check",
        "--json",
    ]);

    let CliAction::Run(CliCommand::Package(PackageCommand::PublishPlan(options))) = action else {
        panic!("expected package publish-plan command");
    };
    assert_eq!(options.common.root, PathBuf::from("proofs"));
    assert!(options.common.json);
    assert!(options.check);
}

#[test]
fn package_cli_args_parses_high_trust_check_mode() {
    let action = parse(&[
        "package",
        "high-trust",
        "--root=proofs",
        "--release-policy",
        "ci/release.high-trust.json",
        "--release-policy-hash",
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "--runner-policy",
        "ci/runner.high-trust.json",
        "--runner-policy-hash",
        "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "--challenge-runner-policy",
        "ci/runner.challenge.json",
        "--challenge-runner-policy-hash",
        "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        "--checker-registry",
        "ci/checker-binaries.json",
        "--out",
        "proofs/generated/verified-high-trust.json",
        "--check",
        "--json",
    ]);

    let CliAction::Run(CliCommand::Package(PackageCommand::HighTrust(options))) = action else {
        panic!("expected package high-trust command");
    };
    assert_eq!(options.common.root, PathBuf::from("proofs"));
    assert!(options.common.json);
    assert!(options.check);
    assert_eq!(
        options.release_policy,
        PathBuf::from("ci/release.high-trust.json")
    );
    assert_eq!(
        options.challenge_runner_policy,
        PathBuf::from("ci/runner.challenge.json")
    );
    assert_eq!(
        options.out.as_ref().unwrap(),
        &PathBuf::from("proofs/generated/verified-high-trust.json")
    );
}

#[test]
fn package_cli_args_defaults_verify_certs_checker_to_reference() {
    let action = parse(&["package", "verify-certs"]);

    let CliAction::Run(CliCommand::Package(PackageCommand::VerifyCerts(options))) = action else {
        panic!("expected package verify-certs command");
    };
    assert_eq!(options.checker, PackageChecker::Reference);
    assert_eq!(options.audit_cache, PackageAuditCacheMode::Off);
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
    assert_eq!(options.audit_cache, PackageAuditCacheMode::Off);
    assert_eq!(options.common.root, PathBuf::from("proofs"));
}

#[test]
fn package_verify_certs_audit_cache_args_parse_read_through() {
    let action = parse(&[
        "package",
        "verify-certs",
        "--checker=fast",
        "--audit-cache",
        "read-through",
    ]);

    let CliAction::Run(CliCommand::Package(PackageCommand::VerifyCerts(options))) = action else {
        panic!("expected package verify-certs command");
    };
    assert_eq!(options.checker, PackageChecker::Fast);
    assert_eq!(options.audit_cache, PackageAuditCacheMode::ReadThrough);

    let action = parse(&["package", "verify-certs", "--audit-cache=off"]);
    let CliAction::Run(CliCommand::Package(PackageCommand::VerifyCerts(options))) = action else {
        panic!("expected package verify-certs command");
    };
    assert_eq!(options.audit_cache, PackageAuditCacheMode::Off);
}

#[test]
fn package_verify_certs_audit_cache_args_reject_duplicate_unknown_and_external() {
    let duplicate = parse_error(&[
        "package",
        "verify-certs",
        "--audit-cache",
        "off",
        "--audit-cache=read-through",
    ]);
    assert_eq!(duplicate.reason, UsageReason::DuplicateFlag);
    assert_eq!(duplicate.flag.as_deref(), Some("--audit-cache"));

    let unknown = parse_error(&["package", "verify-certs", "--audit-cache=local-hit"]);
    assert_eq!(unknown.reason, UsageReason::UnsupportedAuditCacheMode);
    assert_eq!(unknown.flag.as_deref(), Some("--audit-cache"));
    assert_eq!(unknown.value.as_deref(), Some("local-hit"));

    let external = parse_error(&[
        "package",
        "verify-certs",
        "--checker",
        "external",
        "--audit-cache",
        "read-through",
        "--runner-policy",
        "ci/runner.release.json",
        "--runner-policy-hash",
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "--checker-registry",
        "ci/checker-binaries.json",
    ]);
    assert_eq!(external.reason, UsageReason::UnsupportedFlag);
    assert_eq!(external.flag.as_deref(), Some("--audit-cache"));
    assert_eq!(external.value.as_deref(), Some("read-through"));
}

#[test]
fn package_cli_args_parses_verify_certs_external_checker_with_runner_inputs() {
    let action = parse(&[
        "package",
        "verify-certs",
        "--checker",
        "external",
        "--runner-policy",
        "ci/runner.release.json",
        "--runner-policy-hash",
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "--checker-registry",
        "ci/checker-binaries.json",
    ]);

    let CliAction::Run(CliCommand::Package(PackageCommand::VerifyCerts(options))) = action else {
        panic!("expected package verify-certs command");
    };
    assert_eq!(options.checker, PackageChecker::External);
    let external = options.external.as_ref().unwrap();
    assert_eq!(
        external.runner_policy,
        PathBuf::from("ci/runner.release.json")
    );
    assert_eq!(
        external.runner_policy_hash,
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    );
    assert_eq!(
        external.checker_registry,
        PathBuf::from("ci/checker-binaries.json")
    );
}

#[test]
fn package_cli_args_rejects_external_checker_without_runner_inputs() {
    let error = parse_error(&["package", "verify-certs", "--checker", "external"]);

    assert_eq!(error.reason, UsageReason::MissingRequiredFlag);
    assert_eq!(error.flag.as_deref(), Some("--runner-policy"));
    assert!(error.value.is_none());
}

#[test]
fn package_cli_args_rejects_high_trust_without_required_inputs() {
    let error = parse_error(&["package", "high-trust", "--check"]);

    assert_eq!(error.reason, UsageReason::MissingRequiredFlag);
    assert_eq!(error.flag.as_deref(), Some("--release-policy"));
    assert!(error.value.is_none());
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
        "--latest",
        "--latest=true",
        "--upload",
        "--upload=true",
        "--sign",
        "--sign=true",
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

    let runner_policy_error = parse_error(&["package", "check", "--runner-policy=ci/policy.json"]);
    assert_eq!(runner_policy_error.reason, UsageReason::UnsupportedFlag);
    assert_eq!(
        runner_policy_error.flag.as_deref(),
        Some("--runner-policy=ci/policy.json")
    );
}

#[test]
fn package_cli_args_rejects_unknown_commands_and_flags() {
    let command_error = parse_error(&["package", "publish"]);
    assert_eq!(command_error.reason, UsageReason::UnknownCommand);
    assert_eq!(command_error.command.as_deref(), Some("package publish"));

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

    let index_error = parse_error(&["package", "index", "--check", "--check"]);
    assert_eq!(index_error.reason, UsageReason::DuplicateFlag);
    assert_eq!(index_error.flag.as_deref(), Some("--check"));

    let publish_plan_error = parse_error(&["package", "publish-plan", "--check", "--check"]);
    assert_eq!(publish_plan_error.reason, UsageReason::DuplicateFlag);
    assert_eq!(publish_plan_error.flag.as_deref(), Some("--check"));
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
    assert_eq!(
        parse(&["package", "index", "--help"]),
        CliAction::Help(HelpTopic::PackageIndex)
    );
    assert_eq!(
        parse(&["package", "publish-plan", "--help"]),
        CliAction::Help(HelpTopic::PackagePublishPlan)
    );
    assert_eq!(
        parse(&["package", "high-trust", "--help"]),
        CliAction::Help(HelpTopic::PackageHighTrust)
    );
}

#[test]
fn package_cli_args_parses_version_topics() {
    assert_eq!(parse(&["--version"]), CliAction::Version);
    assert_eq!(parse(&["-V"]), CliAction::Version);
    assert_eq!(parse(&["version"]), CliAction::Version);
}

#[test]
fn package_cli_args_binary_reports_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_npa"))
        .arg("--version")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("npa {}\n", env!("CARGO_PKG_VERSION"))
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
        "package verify-certs: failed\nerror Usage missing_required_flag field=--runner-policy\n"
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
