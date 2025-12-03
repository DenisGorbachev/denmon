use Command::*;
use clap::Parser;
use derive_more::{Error, From};
use fmt_derive::Display;

#[derive(Parser, Clone, Debug)]
pub enum Command {
    CheckTetherSupply(CheckTetherSupplyCommand),
}

impl Command {
    pub async fn run(self) -> Result<(), CommandRunError> {
        match self {
            CheckTetherSupply(command) => command.run().await.map_err(From::from),
        }
    }
}

#[derive(Error, Display, From, Debug)]
pub enum CommandRunError {
    CheckTetherSupplyCommandRunFailed { source: CheckTetherSupplyCommandRunError },
}

mod check_tether_supply_command;

pub use check_tether_supply_command::*;
