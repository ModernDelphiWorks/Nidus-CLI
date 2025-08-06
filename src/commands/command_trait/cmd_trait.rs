use super::super::super::dto::config_global_dto::ConfigGlobalDTO;
use clap::{Arg, Command};

pub trait ICommand {
    fn new() -> Self;
    fn command() -> Command;
    fn arg() -> Arg;
    fn execute(global_dto: &mut ConfigGlobalDTO, matches: &clap::ArgMatches);
}
