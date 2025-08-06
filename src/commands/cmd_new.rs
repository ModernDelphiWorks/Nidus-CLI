use super::super::core::core_utils::utils;
use super::super::dto::cmd_new_dto::CommandNewDTO;
use super::super::dto::config_global_dto::ConfigGlobalDTO;
use super::command_trait::cmd_trait::ICommand;

use clap::{Arg, ArgAction, Command};
use colored::Colorize;
use std::path::PathBuf;

pub struct CommandNew;

impl ICommand for CommandNew {
    fn new() -> Self {
        Self
    }

    fn arg() -> Arg {
        Arg::new("")
    }

    fn command() -> Command {
        Command::new("new")
            .about("Create a new Nest4d project")
            .arg(
                Arg::new("project_name")
                    .long("new")
                    .value_name("STRING")
                    .help("Name of the project to be created")
                    .required(true),
            )
            .arg(
                Arg::new("path")
                    .long("path")
                    .value_name("STRING")
                    .default_value("./")
                    .value_parser(clap::value_parser!(PathBuf))
                    .help("Target path to create the project in"),
            )
            .arg(
                Arg::new("with-tests")
                    .long("with-tests")
                    .action(ArgAction::SetTrue)
                    .help("Includes the test/ folder"),
            )
    }

    fn execute(_global_dto: &mut ConfigGlobalDTO, _matches: &clap::ArgMatches) {
        let path: PathBuf = _matches.get_one::<PathBuf>("path").unwrap().clone();
        let project_name: String = _matches.get_one::<String>("project_name").unwrap().clone();
        let include_tests: bool = _matches.get_flag("with-tests");

        // Valida path
        if !path.starts_with("./") {
            eprintln!(
                "{} {} {}{}{} {}",
                "Error:".red(),
                "Invalid path format for".red(),
                "(".red(),
                path.to_string_lossy().red(),
                ")".red(),
                "The path must start with './'".red()
            );
            std::process::exit(1);
        }

        // Salva DTO, mesmo que drivers não sejam mais usados
        let command_new = CommandNewDTO::new(path.clone(), project_name.clone());
        _global_dto.set_command_new(command_new);

        // Gera estrutura de projeto
        if let Err(err) =
            utils::generate_project_structure(path.clone(), &project_name, include_tests)
        {
            utils::println_panic(&[&format!("❌ Error generating project structure: {}", err)]);
        } else {
            println!(
                "{} Project '{}' created successfully at {}",
                "✅".green(),
                project_name.bold(),
                path.join(&project_name).display()
            );
        }
    }
}
