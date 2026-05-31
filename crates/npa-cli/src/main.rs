use std::process::ExitCode;

use npa_cli::args::{parse_cli_args, render_help, CliAction};

fn main() -> ExitCode {
    match parse_cli_args(std::env::args().skip(1)) {
        Ok(CliAction::Help(topic)) => {
            println!("{}", render_help(topic));
            ExitCode::SUCCESS
        }
        Ok(CliAction::Run(command)) => {
            eprintln!(
                "error: command execution is not implemented yet for {}",
                command.command_name()
            );
            ExitCode::from(2)
        }
        Err(error) => {
            eprintln!("{}", error.render_human());
            ExitCode::from(2)
        }
    }
}
