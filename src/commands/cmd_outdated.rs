use super::super::dto::config_global_dto::ConfigGlobalDTO;
use super::command_trait::cmd_trait::CliCommand;
use crate::core::core_utils::utils;
use clap::{Arg, Command};
use colored::*;
use git2::{FetchOptions, RemoteCallbacks, Repository};

pub struct CommandOutdated;

impl CliCommand for CommandOutdated {
    fn arg() -> Arg {
        Arg::new("")
    }

    fn command() -> Command {
        Command::new("outdated")
            .about("Check if dependencies have new commits (without updating)")
    }

    fn execute(global_dto: &mut ConfigGlobalDTO, _matches: &clap::ArgMatches) {
        let install = match global_dto.get_command_install() {
            Some(c) => c,
            None => {
                eprintln!("{} nidus.json not loaded.", "❌".red());
                return;
            }
        };

        let mainsrc = install.mainsrc.trim_end_matches('/').to_string();
        let download_url = install.download.clone();

        println!("{}", "\nChecking for updates...\n".bold().cyan());

        let mut up_to_date = 0usize;
        let mut outdated = 0usize;
        let mut errors = 0usize;

        let dependencies: Vec<(String, String)> = install
            .dependencies
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        for (url, branch) in &dependencies {
            let name = match utils::extract_repo_name(url) {
                Some(n) => n,
                None => continue,
            };
            let dest = format!("{}/{}", mainsrc, name);
            let is_framework = url == &download_url;
            let label = if is_framework {
                format!("[framework] {}", name)
            } else {
                name.clone()
            };

            match check_outdated(&dest, branch) {
                Ok(true) => {
                    println!(
                        "  {} {:<35} {}",
                        "^".yellow(),
                        label.bold(),
                        "outdated -- new commits available".yellow()
                    );
                    outdated += 1;
                }
                Ok(false) => {
                    println!(
                        "  {} {:<35} {}",
                        "ok".green(),
                        label.bold(),
                        "up to date".green()
                    );
                    up_to_date += 1;
                }
                Err(e) => {
                    println!(
                        "  {} {:<35} {}",
                        "??".yellow(),
                        label.bold(),
                        format!("could not check: {}", e).dimmed()
                    );
                    errors += 1;
                }
            }
        }

        println!();
        println!("{}", "Summary".bold().cyan());
        println!("  {} {}", "Up to date:".bold(), up_to_date.to_string().green());
        println!("  {} {}", "Outdated:  ".bold(), outdated.to_string().yellow());
        if errors > 0 {
            println!("  {} {}", "Errors:    ".bold(), errors.to_string().red());
        }
        if outdated > 0 {
            println!(
                "\n  {}",
                "Run `Nidus update` to fast-forward all outdated dependencies.".dimmed()
            );
        }
        println!();
    }
}

/// Fetches the remote tracking ref and compares with local HEAD.
/// Returns `Ok(true)` if there are new commits, `Ok(false)` if up to date.
fn check_outdated(repo_path: &str, branch_hint: &str) -> Result<bool, String> {
    let repo = Repository::open(repo_path)
        .map_err(|e| format!("not cloned or not a git repo: {}", e))?;

    let branch_name = if branch_hint.is_empty() {
        repo.head()
            .map_err(|e| format!("HEAD unreadable: {}", e))?
            .shorthand()
            .unwrap_or("main")
            .to_string()
    } else {
        branch_hint.to_string()
    };

    let mut remote = repo
        .find_remote("origin")
        .map_err(|e| format!("no origin remote: {}", e))?;

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username, allowed| {
        if allowed.is_ssh_key() {
            git2::Cred::ssh_key_from_agent(username.unwrap_or("git"))
        } else {
            git2::Cred::default()
        }
    });

    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);

    let refspec = format!(
        "refs/heads/{}:refs/remotes/origin/{}",
        branch_name, branch_name
    );
    remote
        .fetch(&[&refspec], Some(&mut fetch_opts), None)
        .map_err(|e| format!("fetch failed: {}", e))?;

    let local_oid = repo
        .head()
        .and_then(|h| h.peel_to_commit())
        .map(|c| c.id())
        .map_err(|e| format!("local HEAD unreadable: {}", e))?;

    let tracking_ref = format!("refs/remotes/origin/{}", branch_name);
    let remote_oid = repo
        .find_reference(&tracking_ref)
        .and_then(|r| r.peel_to_commit())
        .map(|c| c.id())
        .map_err(|e| format!("remote ref unreadable: {}", e))?;

    Ok(local_oid != remote_oid)
}
