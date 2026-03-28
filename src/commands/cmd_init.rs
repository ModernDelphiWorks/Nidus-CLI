use super::command_trait::cmd_trait::CliCommand;
use super::super::dto::config_global_dto::ConfigGlobalDTO;
use crate::validation::validate_git_url;
use clap::{Arg, ArgAction, Command};
use colored::*;
use std::path::Path;

const DEFAULT_NIDUS_JSON: &str = r#"{
  "name": "Nidus",
  "description": "Nidus Framework for Delphi",
  "version": "master",
  "homepage": "https://www.isaquepinheiro.com.br/nidus",
  "mainsrc": "src/",
  "projects": [],
  "download": "https://github.com/ModernDelphiWorks/Nidus.git",
  "dependencies": {
    "https://github.com/HashLoad/Horse.git": "",
    "https://github.com/ModernDelphiWorks/ModernSyntax.git": "",
    "https://github.com/ModernDelphiWorks/InjectContainer.git": ""
  }
}
"#;

pub struct CommandInit;

impl CliCommand for CommandInit {
    fn arg() -> Arg {
        Arg::new("")
    }

    fn command() -> Command {
        Command::new("init")
            .about("🗂  Initialize nidus.json in an existing Delphi project")
            .long_about(
                "Creates a nidus.json configuration file in the current directory.\n\
                 Unlike `Nidus new`, this does NOT scaffold any source files — it only\n\
                 creates the config so you can run `Nidus install` afterwards.\n\n\
                 Use --download to set the main framework URL (default: Nidus).\n\
                 Use --mainsrc  to set the sources directory (default: src/).",
            )
            .arg(
                Arg::new("download")
                    .long("download")
                    .value_name("URL")
                    .help("Main framework git URL (default: Nidus official repo)"),
            )
            .arg(
                Arg::new("mainsrc")
                    .long("mainsrc")
                    .value_name("DIR")
                    .help("Sources directory relative to the project root (default: src/)"),
            )
            .arg(
                Arg::new("force")
                    .long("force")
                    .short('f')
                    .action(ArgAction::SetTrue)
                    .help("Overwrite nidus.json if it already exists"),
            )
    }

    fn execute(_global_dto: &mut ConfigGlobalDTO, matches: &clap::ArgMatches) {
        let force = matches.get_flag("force");

        if Path::new("nidus.json").exists() && !force {
            eprintln!(
                "{} nidus.json already exists. Use --force to overwrite.",
                "❌".red()
            );
            std::process::exit(1);
        }

        // Customise the default JSON when optional flags are provided
        let download = matches
            .get_one::<String>("download")
            .map(String::as_str)
            .unwrap_or("https://github.com/ModernDelphiWorks/Nidus.git");

        if let Err(e) = validate_git_url(download) {
            eprintln!("{} Invalid --download URL: {}", "❌".red(), e);
            std::process::exit(1);
        }

        let mainsrc = matches
            .get_one::<String>("mainsrc")
            .map(String::as_str)
            .unwrap_or("src/");

        // Build JSON — swap defaults if the user provided custom values
        let json = DEFAULT_NIDUS_JSON
            .replace(
                "\"https://github.com/ModernDelphiWorks/Nidus.git\"",
                &format!("\"{}\"", download),
            )
            .replace("\"src/\"", &format!("\"{}\"", mainsrc));

        match std::fs::write("nidus.json", &json) {
            Ok(_) => {
                println!("{}", "\n✅ nidus.json created successfully.\n".bold().green());
                println!("  {} {}", "download:".bold(), download.cyan());
                println!("  {} {}", "mainsrc: ".bold(), mainsrc.cyan());
                println!();
                println!(
                    "{}",
                    "💡 Run `Nidus install` to clone the dependencies.".dimmed()
                );
            }
            Err(e) => {
                eprintln!("{} Could not write nidus.json: {}", "❌".red(), e);
                std::process::exit(1);
            }
        }
    }
}
