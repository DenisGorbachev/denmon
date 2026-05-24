use Subcommand::*;
use std::process::ExitCode;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
#[command(author, version, about, propagate_version = true)]
pub struct Command {
    #[command(subcommand)]
    subcommand: Subcommand,
}

#[derive(clap::Subcommand, Clone, Debug)]
pub enum Subcommand {
    CheckTetherSupply(CheckTetherSupplyCommand),
}

impl Command {
    pub async fn run(self) -> Result<ExitCode, CommandRunError> {
        use CommandRunError::*;
        let Self {
            subcommand,
        } = self;
        match subcommand {
            CheckTetherSupply(command) => command
                .run()
                .await
                .map(|()| ExitCode::SUCCESS)
                .map_err(|source| CheckTetherSupplyCommandRunFailed {
                    source,
                }),
        }
    }
}

#[derive(Error, Debug)]
pub enum CommandRunError {
    #[error("failed to run check-tether-supply command")]
    CheckTetherSupplyCommandRunFailed { source: CheckTetherSupplyCommandRunError },
}

mod check_tether_supply_command;

pub use check_tether_supply_command::*;
