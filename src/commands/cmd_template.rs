//! Wrapper para o comando template seguindo a arquitetura existente

use crate::commands::command_trait::cmd_trait::CliCommand;
use crate::commands::template::{TemplateAction, TemplateCommand};
use crate::dto::config_global_dto::ConfigGlobalDTO;
use clap::{Arg, ArgMatches, Command};
use colored::Colorize;

/// Comando template seguindo a interface CliCommand
pub struct CommandTemplate;

impl CliCommand for CommandTemplate {
    fn arg() -> Arg {
        Arg::new("template")
            .help("Template management commands")
            .action(clap::ArgAction::SetTrue)
    }

    fn command() -> Command {
        Command::new("template")
            .about("🎨 Manage custom templates")
            .long_about("Manage custom templates for Nidus projects and modules")
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
                    .about("Install external template [not yet implemented]")
                    .long_about(
                        "Install an external template from a Git repository.\n\
                         \n\
                         NOT YET IMPLEMENTED. Install manually:\n\
                         \n\
                         1. Clone the template repository into ~/.Nidus/templates/<name>/\n\
                         2. Ensure the directory contains a valid template.json\n\
                         \n\
                         Example:\n  git clone https://github.com/user/my-template ~/.Nidus/templates/my-template",
                    )
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
                    .about("Update installed templates [not yet implemented]")
                    .long_about(
                        "Update external templates installed in ~/.Nidus/templates/.\n\
                         \n\
                         NOT YET IMPLEMENTED. Update manually:\n\
                         \n  cd ~/.Nidus/templates/<name> && git pull\n\
                         \n\
                         To update ALL installed templates:\n\
                         \n  for d in ~/.Nidus/templates/*/; do git -C \"$d\" pull; done",
                    )
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
            .subcommand(
                Command::new("publish")
                    .about("📤 Publish a local template to a git remote")
                    .arg(Arg::new("name").required(true).help("Template name"))
                    .arg(Arg::new("url").required(true).help("Remote git URL (e.g. https://github.com/user/my-template.git)"))
                    .arg_required_else_help(true),
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
                    .map(std::path::PathBuf::from);
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
                    .map(std::path::PathBuf::from);
                let template_cmd = TemplateCommand {
                    action: TemplateAction::Test { name, output },
                };
                template_cmd.execute()
            }
            Some(("export", sub_matches)) => {
                let name = sub_matches.get_one::<String>("name").cloned();
                let output = sub_matches
                    .get_one::<String>("output")
                    .map(std::path::PathBuf::from);
                let force = sub_matches.get_flag("force");
                let template_cmd = TemplateCommand {
                    action: TemplateAction::Export { name, output, force },
                };
                template_cmd.execute()
            }
            Some(("publish", sub_matches)) => {
                Self::publish_template(sub_matches);
                return;
            }
            _ => {
                println!(
                    "{} Use 'Nidus template --help' for usage information",
                    "Error:".red().bold()
                );
                return;
            }
        };

        if let Err(e) = result {
            crate::core::core_utils::utils::handle_error(e);
        }
    }
}

impl CommandTemplate {
    fn publish_template(matches: &ArgMatches) {
        use crate::core::core_utils::utils;
        use crate::validation::validate_git_url;

        let name = matches.get_one::<String>("name").unwrap();
        let url = matches.get_one::<String>("url").unwrap();

        if let Err(e) = validate_git_url(url) {
            eprintln!("{} Invalid URL: {}", "❌".red(), e);
            return;
        }

        let templates_dir = match utils::get_templates_directory() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("{} {}", "❌".red(), e);
                return;
            }
        };

        let template_dir = templates_dir.join(name);
        if !template_dir.exists() {
            eprintln!(
                "{} Template '{}' not found at {}",
                "❌".red(),
                name,
                template_dir.display()
            );
            return;
        }

        let repo = if template_dir.join(".git").exists() {
            match git2::Repository::open(&template_dir) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{} Cannot open repo: {}", "❌".red(), e);
                    return;
                }
            }
        } else {
            match git2::Repository::init(&template_dir) {
                Ok(r) => {
                    println!("{}", "  📁 Initialized git repository".dimmed());
                    r
                }
                Err(e) => {
                    eprintln!("{} Cannot init repo: {}", "❌".red(), e);
                    return;
                }
            }
        };

        let mut index = match repo.index() {
            Ok(i) => i,
            Err(e) => {
                eprintln!("{} Index error: {}", "❌".red(), e);
                return;
            }
        };
        if let Err(e) = index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None) {
            eprintln!("{} Stage error: {}", "❌".red(), e);
            return;
        }
        if let Err(e) = index.write() {
            eprintln!("{} Index write error: {}", "❌".red(), e);
            return;
        }

        let tree_oid = match index.write_tree() {
            Ok(oid) => oid,
            Err(e) => {
                eprintln!("{} Tree error: {}", "❌".red(), e);
                return;
            }
        };
        let tree = repo.find_tree(tree_oid).unwrap();
        let sig = git2::Signature::now("Nidus CLI", "nidus@noreply").unwrap();
        let msg = format!("Publish template {}", name);

        let parents: Vec<git2::Commit> = repo
            .head()
            .ok()
            .and_then(|h| h.peel_to_commit().ok())
            .map(|c| vec![c])
            .unwrap_or_default();
        let parent_refs: Vec<&git2::Commit> = parents.iter().collect();

        if let Err(e) = repo.commit(Some("HEAD"), &sig, &sig, &msg, &tree, &parent_refs) {
            eprintln!("{} Commit error: {}", "❌".red(), e);
            return;
        }
        println!("{}", "  ✅ Committed template files".dimmed());

        let _ = repo.remote_delete("origin");
        let mut remote = match repo.remote("origin", url) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("{} Remote error: {}", "❌".red(), e);
                return;
            }
        };

        let branch = repo
            .head()
            .ok()
            .and_then(|h| h.shorthand().map(str::to_string))
            .unwrap_or_else(|| "main".to_string());

        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_url, username, allowed| {
            if allowed.is_ssh_key() {
                git2::Cred::ssh_key_from_agent(username.unwrap_or("git"))
            } else {
                git2::Cred::default()
            }
        });
        let mut push_opts = git2::PushOptions::new();
        push_opts.remote_callbacks(callbacks);

        let refspec = format!("refs/heads/{}:refs/heads/{}", branch, branch);
        match remote.push(&[&refspec], Some(&mut push_opts)) {
            Ok(_) => {
                println!(
                    "\n{} Template '{}' published to {}",
                    "✅".green(),
                    name.bold(),
                    url.cyan()
                );
            }
            Err(e) => {
                eprintln!("{} Push failed: {}", "❌".red(), e);
                eprintln!(
                    "{}",
                    "  💡 Ensure you have push access and the remote exists.".dimmed()
                );
            }
        }
    }
}