use super::super::dto::cmd_gen_dto::{CommandGenerateDTO, GenerateType};
use super::super::dto::config_global_dto::ConfigGlobalDTO;
use super::command_trait::cmd_trait::CliCommand;
use crate::core::core_generate_module::module;
use crate::core::core_utils::utils;
use crate::error::CliError;
use crate::validation::validate_module_name;
use clap::{Arg, ArgAction, ArgMatches, Command};
use colored::*;
use std::path::PathBuf;

pub struct CommandGen;

impl CliCommand for CommandGen {

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
            .arg(
                Arg::new("template")
                    .long("template")
                    .value_name("TEMPLATE_NAME")
                    .help("Use a specific custom template from ~/.Nidus/templates/<name>"),
            )
            .arg(
                Arg::new("dry_run")
                    .long("dry-run")
                    .action(ArgAction::SetTrue)
                    .help("Preview files that would be generated without writing them"),
            )
            .arg(
                Arg::new("interactive")
                    .long("interactive")
                    .short('i')
                    .action(ArgAction::SetTrue)
                    .help("Select components interactively via a menu"),
            )
        };

        Command::new("gen")
            .about("🔧 Generate Nidus structure")
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
            let raw_name = sub.get_one::<String>("name").expect("name is required");
            if let Err(e) = validate_module_name(raw_name) {
                utils::handle_error(e);
            }
            let dto: CommandGenerateDTO = parse_generate_dto(kind.clone(), sub);
            let module_name = dto.get_name();
            let path = dto.get_path().clone();
            let overwrite = dto.overwrite;
            let template_name = dto.template.clone();
            let dry_run = sub.get_flag("dry_run");
            let interactive = sub.get_flag("interactive");
            let src_path = PathBuf::from(path);

            global.set_command_gen(dto);

            let all_components = vec![
                "module",
                "handler",
                "controller",
                "service",
                "repository",
                "interface",
                "infra",
            ];

            let components: Vec<&str> = if interactive {
                use std::io::IsTerminal;
                if !std::io::stdin().is_terminal() {
                    eprintln!(
                        "{} --interactive requires a TTY. In non-interactive environments, specify components directly.",
                        "❌".red()
                    );
                    return;
                }

                let defaults: Vec<bool> = match kind {
                    GenerateType::All | GenerateType::Scaffold => vec![true; all_components.len()],
                    GenerateType::Module => all_components.iter().map(|c| *c == "module" || *c == "handler").collect(),
                    GenerateType::Handler => all_components.iter().map(|c| *c == "handler").collect(),
                    GenerateType::Controller => all_components.iter().map(|c| *c == "controller").collect(),
                    GenerateType::Service => all_components.iter().map(|c| *c == "service").collect(),
                    GenerateType::Repository => all_components.iter().map(|c| *c == "repository").collect(),
                    GenerateType::Interface => all_components.iter().map(|c| *c == "interface").collect(),
                    GenerateType::Infra => all_components.iter().map(|c| *c == "infra").collect(),
                };

                use dialoguer::MultiSelect;
                let selected = MultiSelect::new()
                    .with_prompt("Select components to generate (space to toggle, enter to confirm)")
                    .items(&all_components)
                    .defaults(&defaults)
                    .interact();

                match selected {
                    Ok(indices) => {
                        if indices.is_empty() {
                            eprintln!("{} No components selected. Aborting.", "❌".red());
                            return;
                        }
                        indices.iter().map(|&i| all_components[i]).collect()
                    }
                    Err(_) => {
                        eprintln!("{} Interactive selection cancelled.", "❌".red());
                        return;
                    }
                }
            } else {
                match kind {
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
                }
            };

            match module::generate_module_structure(
                src_path.clone(),
                module_name.as_str(),
                &components,
                overwrite,
                template_name.as_deref(),
                dry_run,
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
                    utils::handle_error(CliError::IoError(err));
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

/// Builds the DTO from the command arguments
fn parse_generate_dto(kind: GenerateType, matches: &ArgMatches) -> CommandGenerateDTO {
    let name = matches
        .get_one::<String>("name")
        .expect("name is required")
        .to_string();

    let path = matches.get_one::<String>("path").map(String::to_string);
    let overwrite = matches.get_flag("overwrite");
    let template = matches.get_one::<String>("template").cloned();

    CommandGenerateDTO::new(kind, name, path, overwrite, template)
}
