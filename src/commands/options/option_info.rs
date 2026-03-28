use super::super::super::{core::core_utils::utils, dto::config_global_dto::ConfigGlobalDTO};
use crate::commands::command_trait::cmd_trait::CliCommand;
use clap::{Arg, ArgAction, Command};
use colored::*;

pub struct CommandInfo;

impl CliCommand for CommandInfo {

    fn arg() -> Arg {
        Arg::new("info")
            .long("info")
            .short('i')
            .action(ArgAction::SetTrue)
            .help("Displays detailed information for the Nidus CLI.")
    }

    fn command() -> Command {
        Command::new("info").about("ℹ️  Show CLI version, config and environment details")
    }

    fn execute(_global_dto: &mut ConfigGlobalDTO, _matches: &clap::ArgMatches) {
        let banner: Vec<String> = vec![
            "".to_string(),
            "  ███    ███  ███████  ███████    ███    ███   ██████      ███████   ███       ███████".to_string(),
            "  ████   ███    ███    ███  ███   ███    ███  ███    ██    ███   ██  ███         ███  ".to_string(),
            "  █████  ███    ███    ███    ██  ███    ███  ███          ███       ███         ███  ".to_string(),
            "  ███ ██ ███    ███    ███    ██  ███    ███   ██████      ███       ███         ███  ".to_string(),
            "  ███  █████    ███    ███    ██  ███    ███        ███    ███       ███         ███  ".to_string(),
            "  ███   ████    ███    ███  ███   ███    ███  ██    ███    ███   ██  ███   ██    ███  ".to_string(),
            "  ███    ███  ███████  ███████     ████████    ██████      ███████   ████████  ███████".to_string(),
            format!("  Version: {}", utils::version()),
            "  Author: Isaque Pinheiro".to_string(),
            "  Email: isaquesp@gmail.com".to_string(),
            "  Github: https://github.com/ModernDelphiWorks/Nidus.git".to_string(),
            "  Documentation: https://www.isaquepinheiro.com.br/docs/nidus".to_string(),
            "  Language: Rust".to_string(),
            "".to_string(),
        ];
        println!("{}", banner.join("\n").yellow());

        // Project summary — only shown when nidus.json is loaded
        if let Some(install) = _global_dto.get_command_install() {
            println!("{}", "📦 Project summary".bold().cyan());
            println!("  {} {}", "Framework:".bold(), install.download.dimmed());
            println!("  {} {}", "Sources dir:".bold(), install.mainsrc.green());

            let dep_count = install.dependencies.len().saturating_sub(1); // exclude download URL
            println!("  {} {}", "Dependencies:".bold(), dep_count.to_string().green());

            if dep_count > 0 {
                for (url, branch) in &install.dependencies {
                    if url == &install.download {
                        continue;
                    }
                    let name = utils::extract_repo_name(url).unwrap_or_else(|| url.clone());
                    if branch.is_empty() {
                        println!("    • {}", name.cyan());
                    } else {
                        println!("    • {} {}", name.cyan(), format!("({})", branch).dimmed());
                    }
                }
            }

            // Count modules on disk
            let modules_dir = std::path::PathBuf::from(&install.mainsrc).join("modules");
            if modules_dir.exists() {
                let module_count = std::fs::read_dir(&modules_dir)
                    .map(|rd| rd.filter_map(|e| e.ok()).filter(|e| e.path().is_dir()).count())
                    .unwrap_or(0);
                println!("  {} {}", "Modules:".bold(), module_count.to_string().green());
            }

            // Check cloned deps
            let cloned: usize = install.dependencies.iter()
                .filter(|(url, _)| *url != &install.download)
                .filter(|(url, _)| {
                    let name = utils::extract_repo_name(url).unwrap_or_default();
                    std::path::Path::new(&format!("{}/{}", install.mainsrc, name)).exists()
                })
                .count();
            if dep_count > 0 {
                println!("  {} {}/{}", "Cloned:".bold(), cloned.to_string().green(), dep_count);
            }
            println!();
        } else {
            println!("{}", "  ℹ️  No nidus.json found — run `Nidus install` to initialize a project.".dimmed());
            println!();
        }
    }
}
