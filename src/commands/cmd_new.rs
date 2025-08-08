use super::super::core::core_utils::utils;
use super::super::dto::cmd_new_dto::CommandNewDTO;
use super::super::dto::config_global_dto::ConfigGlobalDTO;
use super::command_trait::cmd_trait::ICommand;
use crate::core::core_generate_project::project;
use crate::validation::{validate_project_name, validate_project_path};
use crate::error::CliError;
use clap::{Arg, ArgAction, Command};
use colored::Colorize;
use log::{debug, info};
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
            .about("🆕 Create a new Nest4d project")
            .arg(
                Arg::new("project_name")
                    .value_name("NAME")
                    .help("Name of the project to be created")
                    .required(true)
                    .index(1),
            )
            .arg(
                Arg::new("path")
                    .long("path")
                    .value_name("PATH")
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

    fn execute(global_dto: &mut ConfigGlobalDTO, matches: &clap::ArgMatches) {
        let path: PathBuf = matches.get_one::<PathBuf>("path").unwrap().clone();
        let project_name: String = matches.get_one::<String>("project_name").unwrap().clone();
        let include_tests: bool = matches.get_flag("with-tests");

        debug!("Criando projeto: {} em {}", project_name, path.display());

        // Valida nome do projeto
        if let Err(e) = validate_project_name(&project_name) {
            utils::handle_error(e);
        }

        // Valida path
        let path_str = path.to_string_lossy();
        if let Err(e) = validate_project_path(&path_str) {
            utils::handle_error(e);
        }

        // Salva DTO, mesmo que drivers não sejam mais usados
        let command_new: CommandNewDTO = CommandNewDTO::new(path.clone(), project_name.clone());
        global_dto.set_command_new(command_new);

        // Gera estrutura de projeto
        if let Err(err) =
            project::generate_project_structure(path.clone(), &project_name, include_tests)
        {
            let cli_error = CliError::from(err);
            utils::handle_error(cli_error);
        } else {
            info!("Projeto '{}' criado com sucesso em {}", project_name, path.join(&project_name).display());
            println!(
                "{} Project '{}' created successfully at {}",
                "✅".green(),
                project_name.bold(),
                path.join(&project_name).display()
            );
        }
    }
}
