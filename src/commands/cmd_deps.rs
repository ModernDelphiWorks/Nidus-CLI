use super::command_trait::cmd_trait::CliCommand;
use super::super::dto::config_global_dto::ConfigGlobalDTO;
use crate::core::core_utils::utils;
use clap::{Arg, ArgAction, Command};
use colored::*;
use serde::Serialize;
use std::path::Path;

pub struct CommandDeps;

impl CliCommand for CommandDeps {
    fn arg() -> Arg {
        Arg::new("")
    }

    fn command() -> Command {
        Command::new("deps")
            .about("📋 List project dependencies with clone and git status")
            .arg(
                Arg::new("json")
                    .long("json")
                    .action(ArgAction::SetTrue)
                    .help("Output as JSON"),
            )
    }

    fn execute(global_dto: &mut ConfigGlobalDTO, matches: &clap::ArgMatches) {
        let json_mode = matches.get_flag("json");

        let install = match global_dto.get_command_install() {
            Some(c) => c,
            None => {
                eprintln!(
                    "{} nidus.json not found. Run `Nidus init` or `Nidus install` first.",
                    "❌".red()
                );
                std::process::exit(1);
            }
        };

        let mainsrc = install.mainsrc.trim_end_matches('/').to_string();
        let download_url = install.download.clone();

        // Collect dependency info
        let mut entries: Vec<DepEntry> = Vec::new();

        // Framework first
        entries.push(make_entry(&download_url, "", &mainsrc, true));

        // Extra deps
        let mut extra: Vec<DepEntry> = install
            .dependencies
            .iter()
            .filter(|(url, _)| *url != &download_url)
            .map(|(url, branch)| make_entry(url, branch, &mainsrc, false))
            .collect();

        // Sort extra deps by name
        extra.sort_by(|a, b| a.name.cmp(&b.name));
        entries.extend(extra);

        if json_mode {
            print_json(&entries);
        } else {
            print_table(&entries, &mainsrc);
        }
    }
}

#[derive(Debug, Serialize)]
struct DepEntry {
    name: String,
    url: String,
    branch: String,
    cloned: bool,
    has_git: bool,
    last_commit: Option<String>,
    is_framework: bool,
}

fn make_entry(url: &str, branch: &str, mainsrc: &str, is_framework: bool) -> DepEntry {
    let name = utils::extract_repo_name(url).unwrap_or_else(|| url.to_string());
    let dest = format!("{}/{}", mainsrc, name);
    let dest_path = Path::new(&dest);
    let cloned = dest_path.exists();
    let has_git = dest_path.join(".git").is_dir();

    let last_commit = if has_git {
        read_last_commit(&dest)
    } else {
        None
    };

    DepEntry {
        name,
        url: url.to_string(),
        branch: if branch.is_empty() { "default".to_string() } else { branch.to_string() },
        cloned,
        has_git,
        last_commit,
        is_framework,
    }
}

fn read_last_commit(repo_path: &str) -> Option<String> {
    git2::Repository::open(repo_path).ok().and_then(|repo| {
        repo.head().ok().and_then(|head| {
            head.peel_to_commit().ok().map(|commit| {
                let id = commit.id().to_string();
                let short = &id[..7];
                let time = commit.time();
                // Convert git timestamp to date string
                let secs = time.seconds();
                let dt = chrono::DateTime::from_timestamp(secs, 0)
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|| "?".to_string());
                format!("{} ({})", short, dt)
            })
        })
    })
}

fn print_table(entries: &[DepEntry], _mainsrc: &str) {
    println!("{}", "\n📋 Dependencies\n".bold().cyan());

    for entry in entries {
        let label = if entry.is_framework {
            format!("[framework] {}", entry.name)
        } else {
            entry.name.clone()
        };

        let status_icon = if !entry.cloned {
            "❌".to_string()
        } else if !entry.has_git {
            "⚠️ ".to_string()
        } else {
            "✅".to_string()
        };

        let status_text = if !entry.cloned {
            "not cloned".red().to_string()
        } else if !entry.has_git {
            "no .git/".yellow().to_string()
        } else {
            "cloned".green().to_string()
        };

        println!(
            "  {}  {:<30}  {}",
            status_icon,
            label.bold(),
            status_text
        );
        println!("       {} {}", "URL:   ".dimmed(), entry.url.dimmed());
        println!("       {} {}", "Branch:".dimmed(), entry.branch.dimmed());
        if let Some(ref sha) = entry.last_commit {
            println!("       {} {}", "Commit:".dimmed(), sha.dimmed());
        }
        println!();
    }

    let cloned = entries.iter().filter(|e| e.cloned).count();
    let total = entries.len();
    println!(
        "  {} {}/{}",
        "Total cloned:".bold(),
        cloned.to_string().green(),
        total
    );

    let not_cloned = total - cloned;
    if not_cloned > 0 {
        println!(
            "  {}",
            format!("⚠️  {} dep(s) not cloned — run `Nidus install`", not_cloned).yellow()
        );
    }
    println!();
}

fn print_json(entries: &[DepEntry]) {
    match serde_json::to_string_pretty(entries) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("{} Could not serialize deps: {}", "❌".red(), e),
    }
}
