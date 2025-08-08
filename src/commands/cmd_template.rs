//! Wrapper para o comando template seguindo a arquitetura existente

use crate::commands::command_trait::cmd_trait::ICommand;
use crate::commands::template::{TemplateAction, TemplateCommand};
use crate::dto::config_global_dto::ConfigGlobalDTO;
use clap::{Arg, ArgMatches, Command};
use colored::Colorize;

/// Comando template seguindo a interface ICommand
pub struct CommandTemplate;

impl ICommand for CommandTemplate {
    fn new() -> Self {
        CommandTemplate
    }

    fn arg() -> Arg {
        Arg::new("template")
            .help("Template management commands")
            .action(clap::ArgAction::SetTrue)
    }

    fn command() -> Command {
        Command::new("template")
            .about("🎨 Manage custom templates")
            .long_about("Manage custom templates for Nest4D projects and modules")
            .subcommand(
                Command::new("list")
                    .about("List available templates")
                    .arg(
                        Arg::new("favorites")
                            .short('f')
                            .long("favorites")
                            .help("Show only favorite templates")
                            .action(clap::ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("category")
                            .short('c')
                            .long("category")
                            .help("Filter by category")
                            .value_name("CATEGORY"),
                    ),
            )
            .subcommand(
                Command::new("info")
                    .about("Show template information")
                    .arg(
                        Arg::new("name")
                            .help("Template name")
                            .required(true)
                            .value_name("NAME"),
                    ),
            )
            .subcommand(
                Command::new("install")
                    .about("Install external template")
                    .arg(
                        Arg::new("source")
                            .help("Template source URL or name")
                            .required(true)
                            .value_name("SOURCE"),
                    )
                    .arg(
                        Arg::new("name")
                            .short('n')
                            .long("name")
                            .help("Local name for the template")
                            .value_name("NAME"),
                    )
                    .arg(
                        Arg::new("force")
                            .short('f')
                            .long("force")
                            .help("Force reinstallation")
                            .action(clap::ArgAction::SetTrue),
                    ),
            )
            .subcommand(
                Command::new("remove")
                    .about("Remove a template")
                    .arg(
                        Arg::new("name")
                            .help("Template name")
                            .required(true)
                            .value_name("NAME"),
                    )
                    .arg(
                        Arg::new("yes")
                            .short('y')
                            .long("yes")
                            .help("Confirm removal without asking")
                            .action(clap::ArgAction::SetTrue),
                    ),
            )
            .subcommand(
                Command::new("create")
                    .about("Create a new template")
                    .arg(
                        Arg::new("name")
                            .help("Template name")
                            .required(true)
                            .value_name("NAME"),
                    )
                    .arg(
                        Arg::new("description")
                            .short('d')
                            .long("description")
                            .help("Template description")
                            .value_name("DESCRIPTION"),
                    )
                    .arg(
                        Arg::new("from")
                            .short('f')
                            .long("from")
                            .help("Base directory to create template from")
                            .value_name("PATH"),
                    ),
            )
            .subcommand(
                Command::new("config")
                    .about("Configure a template")
                    .arg(
                        Arg::new("name")
                            .help("Template name")
                            .required(true)
                            .value_name("NAME"),
                    )
                    .arg(
                        Arg::new("key")
                            .help("Configuration key")
                            .value_name("KEY"),
                    )
                    .arg(
                        Arg::new("value")
                            .help("Configuration value")
                            .value_name("VALUE"),
                    ),
            )
            .subcommand(
                Command::new("update")
                    .about("Update templates")
                    .arg(
                        Arg::new("name")
                            .help("Specific template name")
                            .value_name("NAME"),
                    )
                    .arg(
                        Arg::new("all")
                            .short('a')
                            .long("all")
                            .help("Update all templates")
                            .action(clap::ArgAction::SetTrue),
                    ),
            )
            .subcommand(
                Command::new("test")
                    .about("Test a template")
                    .arg(
                        Arg::new("name")
                            .help("Template name")
                            .required(true)
                            .value_name("NAME"),
                    )
                    .arg(
                        Arg::new("output")
                            .short('o')
                            .long("output")
                            .help("Output directory for test")
                            .value_name("PATH"),
                    ),
            )
            .subcommand(
                Command::new("export")
                    .about("Export built-in templates to disk")
                    .arg(
                        Arg::new("name")
                            .help("Template name to export (optional, exports all if not specified)")
                            .value_name("NAME"),
                    )
                    .arg(
                        Arg::new("output")
                            .short('o')
                            .long("output")
                            .help("Output directory")
                            .value_name("PATH")
                            .default_value("./templates"),
                    )
                    .arg(
                        Arg::new("force")
                            .short('f')
                            .long("force")
                            .help("Overwrite existing files")
                            .action(clap::ArgAction::SetTrue),
                    ),
            )
    }

    fn execute(_config_global: &mut ConfigGlobalDTO, matches: &ArgMatches) {
        let result = match matches.subcommand() {
            Some(("list", sub_matches)) => {
                let template_cmd = TemplateCommand {
                    action: TemplateAction::List {
                        favorites: sub_matches.get_flag("favorites"),
                        category: sub_matches.get_one::<String>("category").cloned(),
                    },
                };
                template_cmd.execute()
            }
            Some(("info", sub_matches)) => {
                let name = sub_matches.get_one::<String>("name").unwrap().clone();
                let template_cmd = TemplateCommand {
                    action: TemplateAction::Info { name },
                };
                template_cmd.execute()
            }
            Some(("install", sub_matches)) => {
                let source = sub_matches.get_one::<String>("source").unwrap().clone();
                let name = sub_matches.get_one::<String>("name").cloned();
                let force = sub_matches.get_flag("force");
                let template_cmd = TemplateCommand {
                    action: TemplateAction::Install { source, name, force },
                };
                template_cmd.execute()
            }
            Some(("remove", sub_matches)) => {
                let name = sub_matches.get_one::<String>("name").unwrap().clone();
                let yes = sub_matches.get_flag("yes");
                let template_cmd = TemplateCommand {
                    action: TemplateAction::Remove { name, yes },
                };
                template_cmd.execute()
            }
            Some(("create", sub_matches)) => {
                let name = sub_matches.get_one::<String>("name").unwrap().clone();
                let description = sub_matches.get_one::<String>("description").cloned();
                let from = sub_matches
                    .get_one::<String>("from")
                    .map(|s| std::path::PathBuf::from(s));
                let template_cmd = TemplateCommand {
                    action: TemplateAction::Create {
                        name,
                        description,
                        from,
                    },
                };
                template_cmd.execute()
            }
            Some(("config", sub_matches)) => {
                let name = sub_matches.get_one::<String>("name").unwrap().clone();
                let key = sub_matches.get_one::<String>("key").cloned();
                let value = sub_matches.get_one::<String>("value").cloned();
                let template_cmd = TemplateCommand {
                    action: TemplateAction::Config { name, key, value },
                };
                template_cmd.execute()
            }
            Some(("update", sub_matches)) => {
                let name = sub_matches.get_one::<String>("name").cloned();
                let all = sub_matches.get_flag("all");
                let template_cmd = TemplateCommand {
                    action: TemplateAction::Update { name, all },
                };
                template_cmd.execute()
            }
            Some(("test", sub_matches)) => {
                let name = sub_matches.get_one::<String>("name").unwrap().clone();
                let output = sub_matches
                    .get_one::<String>("output")
                    .map(|s| std::path::PathBuf::from(s));
                let template_cmd = TemplateCommand {
                    action: TemplateAction::Test { name, output },
                };
                template_cmd.execute()
            }
            Some(("export", sub_matches)) => {
                let name = sub_matches.get_one::<String>("name").cloned();
                let output = sub_matches
                    .get_one::<String>("output")
                    .map(|s| std::path::PathBuf::from(s));
                let force = sub_matches.get_flag("force");
                let template_cmd = TemplateCommand {
                    action: TemplateAction::Export { name, output, force },
                };
                template_cmd.execute()
            }
            _ => {
                println!(
                    "{} Use 'nest4d template --help' for usage information",
                    "Error:".red().bold()
                );
                return;
            }
        };

        if let Err(e) = result {
            println!("{} {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    }
}