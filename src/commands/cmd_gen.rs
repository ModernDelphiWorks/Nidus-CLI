use super::super::dto::cmd_gen_dto::{CommandGenerateDTO, GenerateType};
use super::super::dto::config_global_dto::ConfigGlobalDTO;
use super::command_trait::cmd_trait::ICommand;
use crate::core::core_generate_module::module;
use clap::{Arg, ArgAction, ArgMatches, Command};
use colored::*;
use std::path::PathBuf;

pub struct CommandGen;

impl ICommand for CommandGen {
    fn new() -> Self {
        Self
    }

    fn arg() -> Arg {
        Arg::new("")
    }

    fn command() -> Command {
        let common_args = |cmd: Command| {
            cmd.arg(
                Arg::new("name")
                    .help("Module name")
                    .required(true)
                    .value_name("MODULE_NAME"),
            )
            .arg(
                Arg::new("flat")
                    .long("flat")
                    .action(ArgAction::SetTrue)
                    .help("Create files in flat structure (not used yet)"),
            )
            .arg(
                Arg::new("path")
                    .long("path")
                    .value_name("PATH")
                    .help("Custom path to src folder (default: ./src)"),
            )
            .arg(
                Arg::new("overwrite")
                    .long("overwrite")
                    .action(ArgAction::SetTrue)
                    .help("Overwrite existing files if they exist"),
            )
        };

        Command::new("gen")
            .about("🔧 Generate Nest4d structure")
            .visible_alias("generate")
            .subcommand(common_args(
                Command::new("module")
                    .about("📦 Generate a module and handler")
                    .arg_required_else_help(true),
            ))
            .subcommand(common_args(
                Command::new("handler")
                    .about("🌐 Generate handler")
                    .arg_required_else_help(true),
            ))
            .subcommand(common_args(
                Command::new("controller")
                    .about("🎯 Generate a controller")
                    .arg_required_else_help(true),
            ))
            .subcommand(common_args(
                Command::new("service")
                    .about("⚙️ Generate a service")
                    .arg_required_else_help(true),
            ))
            .subcommand(common_args(
                Command::new("repository")
                    .about("🗃 Generate a repository")
                    .arg_required_else_help(true),
            ))
            .subcommand(common_args(
                Command::new("interface")
                    .about("🔌 Generate an interface")
                    .arg_required_else_help(true),
            ))
            .subcommand(common_args(
                Command::new("infra")
                    .about("🔧 Generate infrastructure")
                    .arg_required_else_help(true),
            ))
            .subcommand(common_args(
                Command::new("scaffold")
                    .about("🧱 Generate base structure (controller, service, etc)")
                    .arg_required_else_help(true),
            ))
            .subcommand(common_args(
                Command::new("all")
                    .about("🧰 Generate full module and logic")
                    .arg_required_else_help(true),
            ))
    }

    fn execute(global: &mut ConfigGlobalDTO, matches: &ArgMatches) {
        let mut parse_and_execute = |kind: GenerateType, sub: &ArgMatches| {
            let dto: CommandGenerateDTO = parse_generate_dto(kind.clone(), sub);
            let module_name = dto.get_name();
            let path = dto.get_path().clone();
            let src_path = PathBuf::from(path);

            global.set_command_gen(dto);

            let components: Vec<&str> = match kind {
                GenerateType::All => vec!["all"],
                GenerateType::Scaffold => vec![
                    "module",
                    "handler",
                    "controller",
                    "service",
                    "repository",
                    "interface",
                    "infra",
                ],
                GenerateType::Module => vec!["module", "handler"],
                GenerateType::Handler => vec!["handler"],
                GenerateType::Controller => vec!["controller"],
                GenerateType::Service => vec!["service"],
                GenerateType::Repository => vec!["repository"],
                GenerateType::Interface => vec!["interface"],
                GenerateType::Infra => vec!["infra"],
            };

            match module::generate_module_structure(
                src_path.clone(),
                module_name.as_str(),
                &components,
            ) {
                Ok(_) => {
                    println!(
                        "{} Module '{}' generated successfully in {}",
                        "✅".green(),
                        module_name.bold(),
                        src_path
                            .join("modules")
                            .join(module_name.to_lowercase())
                            .display()
                    );
                }
                Err(err) => {
                    eprintln!("{} Failed to generate module: {}", "❌".red(), err);
                }
            }
        };

        match matches.subcommand() {
            Some(("module", sub)) => parse_and_execute(GenerateType::Module, sub),
            Some(("handler", sub)) => parse_and_execute(GenerateType::Handler, sub),
            Some(("scaffold", sub)) => parse_and_execute(GenerateType::Scaffold, sub),
            Some(("controller", sub)) => parse_and_execute(GenerateType::Controller, sub),
            Some(("service", sub)) => parse_and_execute(GenerateType::Service, sub),
            Some(("repository", sub)) => parse_and_execute(GenerateType::Repository, sub),
            Some(("interface", sub)) => parse_and_execute(GenerateType::Interface, sub),
            Some(("infra", sub)) => parse_and_execute(GenerateType::Infra, sub),
            Some(("all", sub)) => parse_and_execute(GenerateType::All, sub),
            _ => {
                eprintln!(
                    "{} Invalid subcommand. Use one of: module, scaffold, controller, etc.",
                    "❌".red()
                );
            }
        }
    }
}

/// Monta o DTO a partir dos argumentos
fn parse_generate_dto(kind: GenerateType, matches: &ArgMatches) -> CommandGenerateDTO {
    let name = matches
        .get_one::<String>("name")
        .expect("name is required")
        .to_string();

    let path = matches.get_one::<String>("path").map(String::to_string);
    let flat = matches.get_flag("flat");
    let overwrite = matches.get_flag("overwrite");

    CommandGenerateDTO::new(kind, name, path, flat, overwrite)
}
