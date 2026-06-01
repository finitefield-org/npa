//! Argument model and parser for the `npa` binary.

use std::fmt;
use std::path::PathBuf;

/// Parsed top-level CLI action.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CliAction {
    /// Execute a parsed command.
    Run(CliCommand),
    /// Render deterministic help for the selected topic.
    Help(HelpTopic),
}

/// Parsed top-level command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CliCommand {
    /// `npa package ...`.
    Package(PackageCommand),
}

impl CliCommand {
    /// Stable command name used in diagnostics.
    pub fn command_name(&self) -> &'static str {
        match self {
            Self::Package(command) => command.command_name(),
        }
    }
}

/// Parsed `npa package` subcommand.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PackageCommand {
    /// `npa package check`.
    Check(PackageCommonOptions),
    /// `npa package build-certs`.
    BuildCerts(PackageBuildCertsOptions),
    /// `npa package axiom-report`.
    AxiomReport(PackageAxiomReportOptions),
    /// `npa package index`.
    Index(PackageIndexOptions),
    /// `npa package verify-certs`.
    VerifyCerts(PackageVerifyCertsOptions),
    /// `npa package check-hashes`.
    CheckHashes(PackageCommonOptions),
    /// `npa package publish-plan`.
    PublishPlan(PackagePublishPlanOptions),
}

impl PackageCommand {
    /// Stable command name used in diagnostics.
    pub fn command_name(&self) -> &'static str {
        match self {
            Self::Check(_) => "package check",
            Self::BuildCerts(_) => "package build-certs",
            Self::AxiomReport(_) => "package axiom-report",
            Self::Index(_) => "package index",
            Self::VerifyCerts(_) => "package verify-certs",
            Self::CheckHashes(_) => "package check-hashes",
            Self::PublishPlan(_) => "package publish-plan",
        }
    }

    /// Common options for the package subcommand.
    pub fn common_options(&self) -> &PackageCommonOptions {
        match self {
            Self::Check(options) | Self::CheckHashes(options) => options,
            Self::BuildCerts(options) => &options.common,
            Self::AxiomReport(options) => &options.common,
            Self::Index(options) => &options.common,
            Self::VerifyCerts(options) => &options.common,
            Self::PublishPlan(options) => &options.common,
        }
    }
}

/// Common options accepted by each package subcommand.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageCommonOptions {
    /// Package root path. Defaults to `.` without parent search.
    pub root: PathBuf,
    /// Whether deterministic JSON output was requested.
    pub json: bool,
}

impl Default for PackageCommonOptions {
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            json: false,
        }
    }
}

/// Options for `package build-certs`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageBuildCertsOptions {
    /// Common package command options.
    pub common: PackageCommonOptions,
    /// Check mode: rebuild in memory without writing files.
    pub check: bool,
}

/// Options for `package axiom-report`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageAxiomReportOptions {
    /// Common package command options.
    pub common: PackageCommonOptions,
    /// Check mode: regenerate in memory without writing files.
    pub check: bool,
}

/// Options for `package index`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageIndexOptions {
    /// Common package command options.
    pub common: PackageCommonOptions,
    /// Check mode: regenerate in memory without writing files.
    pub check: bool,
}

/// Options for `package publish-plan`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackagePublishPlanOptions {
    /// Common package command options.
    pub common: PackageCommonOptions,
    /// Check mode: regenerate in memory without writing files.
    pub check: bool,
}

/// Options for `package verify-certs`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageVerifyCertsOptions {
    /// Common package command options.
    pub common: PackageCommonOptions,
    /// Checker mode selected for source-free verification.
    pub checker: PackageChecker,
    /// Required external checker runner inputs when `checker = external`.
    pub external: Option<PackageExternalCheckerOptions>,
}

/// Options required by `package verify-certs --checker external`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageExternalCheckerOptions {
    /// Package-relative runner policy path.
    pub runner_policy: PathBuf,
    /// Expected canonical runner policy hash.
    pub runner_policy_hash: String,
    /// Package-relative checker binary registry path.
    pub checker_registry: PathBuf,
}

/// Supported package certificate checker modes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PackageChecker {
    /// CLR-03 source-free reference checker path.
    Reference,
    /// CLR-03 fast kernel verifier path for local development.
    Fast,
    /// CLR-08 external checker runner path.
    External,
}

impl PackageChecker {
    /// Stable CLI spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reference => "reference",
            Self::Fast => "fast",
            Self::External => "external",
        }
    }
}

/// Help topic selected by `--help`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HelpTopic {
    /// Top-level `npa` help.
    Root,
    /// `npa package` help.
    Package,
    /// `npa package check --help`.
    PackageCheck,
    /// `npa package build-certs --help`.
    PackageBuildCerts,
    /// `npa package axiom-report --help`.
    PackageAxiomReport,
    /// `npa package index --help`.
    PackageIndex,
    /// `npa package verify-certs --help`.
    PackageVerifyCerts,
    /// `npa package check-hashes --help`.
    PackageCheckHashes,
    /// `npa package publish-plan --help`.
    PackagePublishPlan,
}

/// Stable usage error produced by the argument parser.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CliUsageError {
    /// Machine-readable reason code.
    pub reason: UsageReason,
    /// Command context, when known.
    pub command: Option<String>,
    /// Flag involved in the error, when applicable.
    pub flag: Option<String>,
    /// Value involved in the error, when applicable.
    pub value: Option<String>,
}

impl CliUsageError {
    fn new(reason: UsageReason) -> Self {
        Self {
            reason,
            command: None,
            flag: None,
            value: None,
        }
    }

    fn with_command(mut self, command: impl Into<String>) -> Self {
        self.command = Some(command.into());
        self
    }

    fn with_flag(mut self, flag: impl Into<String>) -> Self {
        self.flag = Some(flag.into());
        self
    }

    fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Deterministic human-readable usage diagnostic.
    pub fn render_human(&self) -> String {
        let mut message = format!("error: {}", self.reason.reason_code());
        if let Some(command) = &self.command {
            message.push_str(&format!(" command={command}"));
        }
        if let Some(flag) = &self.flag {
            message.push_str(&format!(" flag={flag}"));
        }
        if let Some(value) = &self.value {
            message.push_str(&format!(" value={value}"));
        }
        message
    }
}

impl fmt::Display for CliUsageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render_human())
    }
}

impl std::error::Error for CliUsageError {}

/// Stable usage reason codes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UsageReason {
    /// Unknown command or subcommand.
    UnknownCommand,
    /// Unknown flag.
    UnknownFlag,
    /// Flag requires a value but none was provided.
    MissingFlagValue,
    /// Flag was provided more than once.
    DuplicateFlag,
    /// A selected mode requires a flag that was not provided.
    MissingRequiredFlag,
    /// Known flag is outside CLR-04 scope or the selected command.
    UnsupportedFlag,
    /// Checker mode is outside CLR-04 scope.
    UnsupportedChecker,
}

impl UsageReason {
    /// Stable reason code used by later structured diagnostics.
    pub fn reason_code(self) -> &'static str {
        match self {
            Self::UnknownCommand => "unknown_command",
            Self::UnknownFlag => "unknown_flag",
            Self::MissingFlagValue => "missing_flag_value",
            Self::DuplicateFlag => "duplicate_flag",
            Self::MissingRequiredFlag => "missing_required_flag",
            Self::UnsupportedFlag => "unsupported_flag",
            Self::UnsupportedChecker => "unsupported_checker",
        }
    }
}

/// Parse `npa` arguments, excluding the binary name.
pub fn parse_cli_args<I, S>(args: I) -> Result<CliAction, CliUsageError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args = args.into_iter().map(Into::into).collect::<Vec<_>>();
    if args.is_empty() {
        return Ok(CliAction::Help(HelpTopic::Root));
    }

    match args[0].as_str() {
        "--help" | "-h" => Ok(CliAction::Help(HelpTopic::Root)),
        "package" => parse_package_args(&args[1..]),
        command => Err(CliUsageError::new(UsageReason::UnknownCommand).with_command(command)),
    }
}

fn parse_package_args(args: &[String]) -> Result<CliAction, CliUsageError> {
    if args.is_empty() {
        return Ok(CliAction::Help(HelpTopic::Package));
    }
    match args[0].as_str() {
        "--help" | "-h" => Ok(CliAction::Help(HelpTopic::Package)),
        "check" => parse_package_check_args(&args[1..]),
        "build-certs" => parse_package_build_certs_args(&args[1..]),
        "axiom-report" => parse_package_axiom_report_args(&args[1..]),
        "index" => parse_package_index_args(&args[1..]),
        "verify-certs" => parse_package_verify_certs_args(&args[1..]),
        "check-hashes" => parse_package_check_hashes_args(&args[1..]),
        "publish-plan" => parse_package_publish_plan_args(&args[1..]),
        command if command.starts_with('-') => {
            Err(flag_error(command, UsageReason::UnknownFlag).with_command("package"))
        }
        command => Err(CliUsageError::new(UsageReason::UnknownCommand)
            .with_command(format!("package {command}"))),
    }
}

fn parse_package_check_args(args: &[String]) -> Result<CliAction, CliUsageError> {
    if contains_help(args) {
        return Ok(CliAction::Help(HelpTopic::PackageCheck));
    }
    let common = parse_common_options(args, "package check", &[])?;
    Ok(CliAction::Run(CliCommand::Package(PackageCommand::Check(
        common,
    ))))
}

fn parse_package_check_hashes_args(args: &[String]) -> Result<CliAction, CliUsageError> {
    if contains_help(args) {
        return Ok(CliAction::Help(HelpTopic::PackageCheckHashes));
    }
    let common = parse_common_options(args, "package check-hashes", &[])?;
    Ok(CliAction::Run(CliCommand::Package(
        PackageCommand::CheckHashes(common),
    )))
}

fn parse_package_build_certs_args(args: &[String]) -> Result<CliAction, CliUsageError> {
    if contains_help(args) {
        return Ok(CliAction::Help(HelpTopic::PackageBuildCerts));
    }

    let mut common_tokens = Vec::new();
    let mut check = false;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--check" => {
                if check {
                    return Err(flag_error("--check", UsageReason::DuplicateFlag)
                        .with_command("package build-certs"));
                }
                check = true;
                index += 1;
            }
            token => {
                common_tokens.push(token.to_owned());
                index += 1;
            }
        }
    }

    let common = parse_common_options(&common_tokens, "package build-certs", &["--check"])?;
    Ok(CliAction::Run(CliCommand::Package(
        PackageCommand::BuildCerts(PackageBuildCertsOptions { common, check }),
    )))
}

fn parse_package_axiom_report_args(args: &[String]) -> Result<CliAction, CliUsageError> {
    if contains_help(args) {
        return Ok(CliAction::Help(HelpTopic::PackageAxiomReport));
    }

    let mut common_tokens = Vec::new();
    let mut check = false;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--check" => {
                if check {
                    return Err(flag_error("--check", UsageReason::DuplicateFlag)
                        .with_command("package axiom-report"));
                }
                check = true;
                index += 1;
            }
            token => {
                common_tokens.push(token.to_owned());
                index += 1;
            }
        }
    }

    let common = parse_common_options(
        &common_tokens,
        "package axiom-report",
        &["--check", "--checker"],
    )?;
    Ok(CliAction::Run(CliCommand::Package(
        PackageCommand::AxiomReport(PackageAxiomReportOptions { common, check }),
    )))
}

fn parse_package_index_args(args: &[String]) -> Result<CliAction, CliUsageError> {
    if contains_help(args) {
        return Ok(CliAction::Help(HelpTopic::PackageIndex));
    }

    let mut common_tokens = Vec::new();
    let mut check = false;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--check" => {
                if check {
                    return Err(flag_error("--check", UsageReason::DuplicateFlag)
                        .with_command("package index"));
                }
                check = true;
                index += 1;
            }
            token => {
                common_tokens.push(token.to_owned());
                index += 1;
            }
        }
    }

    let common = parse_common_options(&common_tokens, "package index", &["--check", "--checker"])?;
    Ok(CliAction::Run(CliCommand::Package(PackageCommand::Index(
        PackageIndexOptions { common, check },
    ))))
}

fn parse_package_publish_plan_args(args: &[String]) -> Result<CliAction, CliUsageError> {
    if contains_help(args) {
        return Ok(CliAction::Help(HelpTopic::PackagePublishPlan));
    }

    let mut common_tokens = Vec::new();
    let mut check = false;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--check" => {
                if check {
                    return Err(flag_error("--check", UsageReason::DuplicateFlag)
                        .with_command("package publish-plan"));
                }
                check = true;
                index += 1;
            }
            token => {
                common_tokens.push(token.to_owned());
                index += 1;
            }
        }
    }

    let common = parse_common_options(&common_tokens, "package publish-plan", &["--check"])?;
    Ok(CliAction::Run(CliCommand::Package(
        PackageCommand::PublishPlan(PackagePublishPlanOptions { common, check }),
    )))
}

fn parse_package_verify_certs_args(args: &[String]) -> Result<CliAction, CliUsageError> {
    if contains_help(args) {
        return Ok(CliAction::Help(HelpTopic::PackageVerifyCerts));
    }

    let mut common_tokens = Vec::new();
    let mut checker = None::<PackageChecker>;
    let mut runner_policy = None::<PathBuf>;
    let mut runner_policy_hash = None::<String>;
    let mut checker_registry = None::<PathBuf>;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--checker" => {
                if checker.is_some() {
                    return Err(flag_error("--checker", UsageReason::DuplicateFlag)
                        .with_command("package verify-certs"));
                }
                let value = flag_value(args, index, "--checker", "package verify-certs")?;
                checker = Some(parse_checker(value)?);
                index += 2;
            }
            "--checker=reference" => {
                if checker.is_some() {
                    return Err(flag_error("--checker", UsageReason::DuplicateFlag)
                        .with_command("package verify-certs"));
                }
                checker = Some(PackageChecker::Reference);
                index += 1;
            }
            "--checker=fast" => {
                if checker.is_some() {
                    return Err(flag_error("--checker", UsageReason::DuplicateFlag)
                        .with_command("package verify-certs"));
                }
                checker = Some(PackageChecker::Fast);
                index += 1;
            }
            "--checker=external" => {
                if checker.is_some() {
                    return Err(flag_error("--checker", UsageReason::DuplicateFlag)
                        .with_command("package verify-certs"));
                }
                checker = Some(PackageChecker::External);
                index += 1;
            }
            token if token.starts_with("--checker=") => {
                if checker.is_some() {
                    return Err(flag_error("--checker", UsageReason::DuplicateFlag)
                        .with_command("package verify-certs"));
                }
                let value = token.trim_start_matches("--checker=");
                if value.is_empty() {
                    return Err(flag_error("--checker", UsageReason::MissingFlagValue)
                        .with_command("package verify-certs"));
                }
                checker = Some(parse_checker(value)?);
                index += 1;
            }
            "--runner-policy" => {
                if runner_policy.is_some() {
                    return Err(flag_error("--runner-policy", UsageReason::DuplicateFlag)
                        .with_command("package verify-certs"));
                }
                let value = flag_value(args, index, "--runner-policy", "package verify-certs")?;
                runner_policy = Some(PathBuf::from(value));
                index += 2;
            }
            token if token.starts_with("--runner-policy=") => {
                if runner_policy.is_some() {
                    return Err(flag_error("--runner-policy", UsageReason::DuplicateFlag)
                        .with_command("package verify-certs"));
                }
                let value = token.trim_start_matches("--runner-policy=");
                if value.is_empty() {
                    return Err(flag_error("--runner-policy", UsageReason::MissingFlagValue)
                        .with_command("package verify-certs"));
                }
                runner_policy = Some(PathBuf::from(value));
                index += 1;
            }
            "--runner-policy-hash" => {
                if runner_policy_hash.is_some() {
                    return Err(
                        flag_error("--runner-policy-hash", UsageReason::DuplicateFlag)
                            .with_command("package verify-certs"),
                    );
                }
                let value =
                    flag_value(args, index, "--runner-policy-hash", "package verify-certs")?;
                runner_policy_hash = Some(value.to_owned());
                index += 2;
            }
            token if token.starts_with("--runner-policy-hash=") => {
                if runner_policy_hash.is_some() {
                    return Err(
                        flag_error("--runner-policy-hash", UsageReason::DuplicateFlag)
                            .with_command("package verify-certs"),
                    );
                }
                let value = token.trim_start_matches("--runner-policy-hash=");
                if value.is_empty() {
                    return Err(
                        flag_error("--runner-policy-hash", UsageReason::MissingFlagValue)
                            .with_command("package verify-certs"),
                    );
                }
                runner_policy_hash = Some(value.to_owned());
                index += 1;
            }
            "--checker-registry" => {
                if checker_registry.is_some() {
                    return Err(flag_error("--checker-registry", UsageReason::DuplicateFlag)
                        .with_command("package verify-certs"));
                }
                let value = flag_value(args, index, "--checker-registry", "package verify-certs")?;
                checker_registry = Some(PathBuf::from(value));
                index += 2;
            }
            token if token.starts_with("--checker-registry=") => {
                if checker_registry.is_some() {
                    return Err(flag_error("--checker-registry", UsageReason::DuplicateFlag)
                        .with_command("package verify-certs"));
                }
                let value = token.trim_start_matches("--checker-registry=");
                if value.is_empty() {
                    return Err(
                        flag_error("--checker-registry", UsageReason::MissingFlagValue)
                            .with_command("package verify-certs"),
                    );
                }
                checker_registry = Some(PathBuf::from(value));
                index += 1;
            }
            token => {
                common_tokens.push(token.to_owned());
                index += 1;
            }
        }
    }

    let common = parse_common_options(
        &common_tokens,
        "package verify-certs",
        &[
            "--checker",
            "--runner-policy",
            "--runner-policy-hash",
            "--checker-registry",
        ],
    )?;
    let checker = checker.unwrap_or(PackageChecker::Reference);
    let has_external_options =
        runner_policy.is_some() || runner_policy_hash.is_some() || checker_registry.is_some();
    let external = if checker == PackageChecker::External {
        Some(PackageExternalCheckerOptions {
            runner_policy: runner_policy.ok_or_else(|| {
                flag_error("--runner-policy", UsageReason::MissingRequiredFlag)
                    .with_command("package verify-certs")
            })?,
            runner_policy_hash: runner_policy_hash.ok_or_else(|| {
                flag_error("--runner-policy-hash", UsageReason::MissingRequiredFlag)
                    .with_command("package verify-certs")
            })?,
            checker_registry: checker_registry.ok_or_else(|| {
                flag_error("--checker-registry", UsageReason::MissingRequiredFlag)
                    .with_command("package verify-certs")
            })?,
        })
    } else {
        if has_external_options {
            let flag = if runner_policy.is_some() {
                "--runner-policy"
            } else if runner_policy_hash.is_some() {
                "--runner-policy-hash"
            } else {
                "--checker-registry"
            };
            return Err(
                flag_error(flag, UsageReason::UnsupportedFlag).with_command("package verify-certs")
            );
        }
        None
    };
    Ok(CliAction::Run(CliCommand::Package(
        PackageCommand::VerifyCerts(PackageVerifyCertsOptions {
            common,
            checker,
            external,
        }),
    )))
}

fn parse_checker(value: &str) -> Result<PackageChecker, CliUsageError> {
    match value {
        "reference" => Ok(PackageChecker::Reference),
        "fast" => Ok(PackageChecker::Fast),
        "external" => Ok(PackageChecker::External),
        other => Err(CliUsageError::new(UsageReason::UnsupportedChecker)
            .with_command("package verify-certs")
            .with_flag("--checker")
            .with_value(other)),
    }
}

fn parse_common_options(
    args: &[String],
    command: &'static str,
    command_flags: &[&str],
) -> Result<PackageCommonOptions, CliUsageError> {
    let mut common = PackageCommonOptions::default();
    let mut root_seen = false;
    let mut json_seen = false;
    let mut index = 0usize;

    while index < args.len() {
        match args[index].as_str() {
            "--root" => {
                if root_seen {
                    return Err(
                        flag_error("--root", UsageReason::DuplicateFlag).with_command(command)
                    );
                }
                let value = flag_value(args, index, "--root", command)?;
                common.root = PathBuf::from(value);
                root_seen = true;
                index += 2;
            }
            token if token.starts_with("--root=") => {
                if root_seen {
                    return Err(
                        flag_error("--root", UsageReason::DuplicateFlag).with_command(command)
                    );
                }
                let value = token.trim_start_matches("--root=");
                if value.is_empty() {
                    return Err(
                        flag_error("--root", UsageReason::MissingFlagValue).with_command(command)
                    );
                }
                common.root = PathBuf::from(value);
                root_seen = true;
                index += 1;
            }
            "--json" => {
                if json_seen {
                    return Err(
                        flag_error("--json", UsageReason::DuplicateFlag).with_command(command)
                    );
                }
                common.json = true;
                json_seen = true;
                index += 1;
            }
            flag if is_unsupported_clr04_flag(flag) || command_flags.contains(&flag) => {
                return Err(flag_error(flag, UsageReason::UnsupportedFlag).with_command(command));
            }
            flag if flag.starts_with('-') => {
                return Err(flag_error(flag, UsageReason::UnknownFlag).with_command(command));
            }
            value => {
                return Err(CliUsageError::new(UsageReason::UnknownCommand)
                    .with_command(format!("{command} {value}")));
            }
        }
    }

    Ok(common)
}

fn flag_value<'a>(
    args: &'a [String],
    index: usize,
    flag: &'static str,
    command: &'static str,
) -> Result<&'a str, CliUsageError> {
    let value = args
        .get(index + 1)
        .ok_or_else(|| flag_error(flag, UsageReason::MissingFlagValue).with_command(command))?;
    if value.starts_with('-') {
        return Err(flag_error(flag, UsageReason::MissingFlagValue).with_command(command));
    }
    Ok(value)
}

fn flag_error(flag: impl Into<String>, reason: UsageReason) -> CliUsageError {
    CliUsageError::new(reason).with_flag(flag)
}

fn contains_help(args: &[String]) -> bool {
    args.iter()
        .any(|argument| argument == "--help" || argument == "-h")
}

fn is_unsupported_clr04_flag(flag: &str) -> bool {
    matches!(
        flag,
        "--changed"
            | "--all"
            | "--registry"
            | "--network"
            | "--latest"
            | "--runner-policy"
            | "--runner-policy-hash"
            | "--checker-registry"
            | "--upload"
            | "--sign"
            | "--update-manifest-hashes"
            | "--include-source"
            | "--include-replay"
            | "--include-ai-traces"
            | "--checker"
    ) || flag.starts_with("--changed=")
        || flag.starts_with("--all=")
        || flag.starts_with("--registry=")
        || flag.starts_with("--network=")
        || flag.starts_with("--latest=")
        || flag.starts_with("--runner-policy=")
        || flag.starts_with("--runner-policy-hash=")
        || flag.starts_with("--checker-registry=")
        || flag.starts_with("--upload=")
        || flag.starts_with("--sign=")
        || flag.starts_with("--update-manifest-hashes=")
        || flag.starts_with("--include-source=")
        || flag.starts_with("--include-replay=")
        || flag.starts_with("--include-ai-traces=")
        || flag.starts_with("--checker=")
}

/// Render deterministic help text.
pub fn render_help(topic: HelpTopic) -> &'static str {
    match topic {
        HelpTopic::Root => {
            "Usage: npa package <command> [options]\n\nCommands:\n  package    Package manifest and certificate commands"
        }
        HelpTopic::Package => {
            "Usage: npa package <command> [options]\n\nCommands:\n  check\n  build-certs\n  axiom-report\n  index\n  verify-certs\n  check-hashes\n  publish-plan\n\nCommon options:\n  --root PATH    Package root, default: .\n  --json         Emit deterministic JSON diagnostics\n  --help         Show help"
        }
        HelpTopic::PackageCheck => {
            "Usage: npa package check [--root PATH] [--json]\n\nValidate npa-package.toml metadata without reading source or certificate artifacts."
        }
        HelpTopic::PackageBuildCerts => {
            "Usage: npa package build-certs [--root PATH] [--json] [--check]\n\nRebuild package certificates. --check writes no files; write mode updates local certificates and generated/package-lock.json."
        }
        HelpTopic::PackageAxiomReport => {
            "Usage: npa package axiom-report [--root PATH] [--json] [--check]\n\nGenerate or check generated/axiom-report.json from source-free package certificate artifacts."
        }
        HelpTopic::PackageIndex => {
            "Usage: npa package index [--root PATH] [--json] [--check]\n\nGenerate or check generated/theorem-index.json from source-free package certificate artifacts."
        }
        HelpTopic::PackageVerifyCerts => {
            "Usage: npa package verify-certs [--root PATH] [--json] [--checker reference|fast|external] [--runner-policy PATH --runner-policy-hash HASH --checker-registry PATH]\n\nVerify certificates through the source-free package verifier. The default checker is reference; external mode requires explicit runner policy and checker registry inputs."
        }
        HelpTopic::PackageCheckHashes => {
            "Usage: npa package check-hashes [--root PATH] [--json]\n\nCheck checked-in package artifact hashes."
        }
        HelpTopic::PackagePublishPlan => {
            "Usage: npa package publish-plan [--root PATH] [--json] [--check]\n\nGenerate or check generated/publish-plan.json from source-free package release metadata."
        }
    }
}
