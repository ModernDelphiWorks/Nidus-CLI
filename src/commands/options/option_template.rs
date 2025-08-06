use super::super::super::dto::config_global_dto::ConfigGlobalDTO;
use crate::commands::command_trait::cmd_trait::ICommand;
use clap::{Arg, ArgAction, Command};
use colored::*;

pub struct CommandTemplate;

impl ICommand for CommandTemplate {
    fn new() -> Self {
        Self
    }

    fn arg() -> Arg {
        Arg::new("templates")
            .long("templates")
            .short('t')
            .action(ArgAction::SetTrue)
            .help("Displays the templates available for the Nest4d CLI")
    }

    fn command() -> Command {
        Command::new("")
    }

    fn execute(_global_dto: &mut ConfigGlobalDTO, _matches: &clap::ArgMatches) {
        let print_text: Vec<String> = vec![
        "".to_string(),
        "|─────────────────────────────────────────────────────────────────────────────────────────────|".to_string(),
        "|        🧩 Templates Disponíveis no Nest4d CLI / Available Templates in Nest4d CLI           |".to_string(),
        "|──────────────────────────|──────────────────────────────────────────────────────────────────|".to_string(),
        "|          Name            | Description                                                      |".to_string(),
        "|──────────────────────────|──────────────────────────────────────────────────────────────────|".to_string(),
        "| controller.pas           | Handles external requests and delegates to services.             |".to_string(),
        "|──────────────────────────|──────────────────────────────────────────────────────────────────|".to_string(),
        "| service.pas              | Business logic layer. Orchestrates application behavior.         |".to_string(),
        "|──────────────────────────|──────────────────────────────────────────────────────────────────|".to_string(),
        "| repository.pas           | Data access layer for persistence (ORM, REST, files, etc).       |".to_string(),
        "|──────────────────────────|──────────────────────────────────────────────────────────────────|".to_string(),
        "| interface.pas            | Contract definitions for mocking and infrastructure impls.       |".to_string(),
        "|──────────────────────────|──────────────────────────────────────────────────────────────────|".to_string(),
        "| infra.pas                | Technical code that implements interfaces (REST, files).         |".to_string(),
        "|──────────────────────────|──────────────────────────────────────────────────────────────────|".to_string(),
        "| module.pas               | Registers module dependencies (DI) and structure for Nest4D.     |".to_string(),
        "|──────────────────────────|──────────────────────────────────────────────────────────────────|".to_string(),
        "| routes.pas               | Defines HTTP endpoints and routes them to controllers.           |".to_string(),
        "|──────────────────────────|──────────────────────────────────────────────────────────────────|".to_string(),
        "".to_string(),
        "✅ Use the command: nest4d gen module <name> to generate all the files above.".to_string(),
        "✅ Use: nest4d gen controller|service|... to generate a specific file individually.".to_string(),
        "".to_string(),
    ];
        println!("{}", &print_text.join("\n").yellow());
    }
}
