use super::super::dto::cmd_init_dto::CommandInitDTO;
use super::super::dto::config_global_dto::ConfigGlobalDTO;
use super::command_trait::cmd_trait::ICommand;
use clap::{Arg, Command};
use colored::*;
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::prelude::*;

pub struct CommandInit;

impl ICommand for CommandInit {
    fn new() -> Self {
        Self
    }

    fn arg() -> Arg {
        Arg::new("")
    }

    fn command() -> Command {
        Command::new("init").about("Prepare the nest4d CLI to start work.")
    }

    fn execute(_global_dto: &mut ConfigGlobalDTO, _matches: &clap::ArgMatches) {
        println!("{}", "✅ Enter package name:".yellow());
        let mut name: String = String::from("Nest4d");
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        if !input.trim().is_empty() {
            name = input.trim().to_string();
        }

        println!("{}", "✅ Enter package description:".yellow());
        let mut description: String = String::from("Nest4d Framework for Delphi");
        let mut input: String = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        if !input.trim().is_empty() {
            description = input.trim().to_string();
        }

        println!("{}", "✅ Enter package version:".yellow());
        let mut version: String = String::from("master");
        let mut input: String = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        if !input.trim().is_empty() {
            version = input.trim().to_string();
        }

        println!("{}", "✅ Enter package homepage:".yellow());
        let mut homepage: String = String::from("https://docs.nest4d.com/");
        let mut input: String = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        if !input.trim().is_empty() {
            homepage = input.trim().to_string();
        }

        println!("{}", "✅ Enter path to main source:".yellow());
        let mut mainsrc: String = String::from("./dependencies");
        let mut input: String = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        if !input.trim().is_empty() {
            mainsrc = input.trim().to_string();
        }

        let mut dependencies: HashMap<String, String> = HashMap::new();
        loop {
            println!(
                "{}:",
                "✅ Enter dependency (or leave blank to finish)".yellow()
            );
            let mut dependency_input: String = String::new();
            io::stdin()
                .read_line(&mut dependency_input)
                .expect("Failed to read input");
            let dependency: String = dependency_input.trim().to_string();
            if dependency.is_empty() {
                break;
            }

            println!("{}", "✅ Enter version for dependency:".cyan());
            let mut version_default: String = String::from("master");
            let mut version_input: String = String::new();
            io::stdin()
                .read_line(&mut version_input)
                .expect("Failed to read input");
            if !version_input.trim().is_empty() {
                version_default = input.trim().to_string();
            }
            let dependency_version: String = version_default.trim().to_string();
            dependencies.insert(dependency, dependency_version);
        }

        let json_dto: CommandInitDTO = CommandInitDTO {
            name: name.trim().to_string(),
            description: description.trim().to_string(),
            version: version.trim().to_string(),
            homepage: homepage.trim().to_string(),
            mainsrc: mainsrc.trim().to_string(),
            projects: Vec::new(),
            dependencies,
        };

        let json: String = serde_json::to_string_pretty(&json_dto)
            .expect("Failed to serialize JSON data to string");
        let mut file: File =
            File::create("nest4d.json").expect("Failed to create file nest4d.json");

        file.write_all(json.as_bytes()).unwrap_or_else(|err| {
            panic!("{}", format!("Failed to write JSON to file: {}", err).red())
        });
        println!(
            "{}",
            "✅ Package info saved to nest4d.json successfully!".green()
        );

        // Após salvar o arquivo JSON
        println!(
            "{}",
            "✅ Package info saved to nest4d.json successfully!".green()
        );

        // Pergunta se deseja instalar agora
        println!();
        println!(
            "{}",
            "❓ Do you want to install dependencies now? (Y/n):".yellow()
        );

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        let answer = input.trim().to_lowercase();
        if answer == "y" || answer == "yes" || answer.is_empty() {
        } else {
            println!("{}", "⏭ Skipped installation.".yellow());
        }
    }
}
