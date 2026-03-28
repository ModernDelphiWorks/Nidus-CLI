use super::super::core::core_add_paths_dproj::dproj;
use super::super::core::core_lockfile::lockfile;
use super::super::core::core_utils::utils;
use super::super::dto::config_global_dto::ConfigGlobalDTO;
use super::command_trait::cmd_trait::CliCommand;
use crate::validation::validate_git_url;
use clap::{Arg, Command};
use colored::*;
use git2::{build::RepoBuilder, FetchOptions, Progress, RemoteCallbacks};
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

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

pub struct CommandInstall;

impl CliCommand for CommandInstall {
    fn arg() -> Arg {
        Arg::new("")
    }

    fn command() -> Command {
        Command::new("install")
            .about("📦 Install the Nidus framework dependencies (git clone into ./dependencies)")
            .long_about("Clones all dependency repositories listed in nidus.json into ./dependencies.\nTo install custom code-generation templates, use: Nidus template install <source>")
            .arg(
                Arg::new("add")
                    .long("add")
                    .value_name("URL")
                    .help("Add a new dependency URL to nidus.json and clone it immediately"),
            )
            .arg(
                Arg::new("branch")
                    .long("branch")
                    .value_name("BRANCH")
                    .help("Branch to use when adding a dependency (used with --add)"),
            )
            .arg(
                Arg::new("remove")
                    .long("remove")
                    .value_name("URL")
                    .help("Remove a dependency URL from nidus.json"),
            )
            .arg(
                Arg::new("frozen")
                    .long("frozen")
                    .action(clap::ArgAction::SetTrue)
                    .help("Fail if nidus.lock is missing or if any cloned repo does not match the locked commit"),
            )
    }

    fn execute(global_dto: &mut ConfigGlobalDTO, matches: &clap::ArgMatches) {
        // ── --frozen: validate cloned repos against nidus.lock ────────────────
        if matches.get_flag("frozen") {
            frozen_check(global_dto);
            return;
        }

        // ── --remove: unregister a dependency from nidus.json ─────────────────
        if let Some(url) = matches.get_one::<String>("remove") {
            if let Err(e) = validate_git_url(url) {
                utils::handle_error(e);
            }
            if global_dto.get_command_install().is_none() {
                eprintln!("{}", "❌ nidus.json not found — nothing to remove from.".red());
                std::process::exit(1);
            }
            match global_dto.remove_dependency(url) {
                Ok(_) => println!("{} Removed '{}' from nidus.json.", "✅".green(), url.bold()),
                Err(e) => utils::handle_error(e),
            }
            return;
        }

        // ── --add: register a new dependency and clone it immediately ──────────
        if let Some(url) = matches.get_one::<String>("add") {
            let branch = matches
                .get_one::<String>("branch")
                .cloned()
                .unwrap_or_default();

            if let Err(e) = validate_git_url(url) {
                utils::handle_error(e);
            }

            // Ensure nidus.json exists before trying to add to it
            if !Path::new("nidus.json").exists() {
                match std::fs::write("nidus.json", DEFAULT_NIDUS_JSON) {
                    Ok(_) => {
                        println!("{}", "📄 nidus.json not found — created with default Nidus dependencies.".cyan());
                    }
                    Err(e) => {
                        eprintln!("{} Could not create nidus.json: {}", "❌".red(), e);
                        return;
                    }
                }
                if let Err(e) = global_dto.reload() {
                    utils::handle_error(e);
                }
            }

            if let Err(e) = global_dto.add_dependency(url.clone(), branch.clone()) {
                utils::handle_error(e);
            }

            println!(
                "{} Added '{}' to nidus.json.",
                "✅".green(),
                url.bold()
            );

            // Clone just this new repository
            let path_src = match global_dto.get_command_install() {
                Some(cmd) => cmd.mainsrc.clone(),
                None => return,
            };
            let repo_name = match utils::extract_repo_name(url) {
                Some(n) => n,
                None => {
                    eprintln!("{} Could not extract repo name from URL.", "❌".red());
                    return;
                }
            };
            let dest = format!("{}/{}", path_src, repo_name);
            if Path::new(&dest).exists() {
                println!("{} {} already exists — skipping clone.", "🔁".blue(), repo_name.blue());
            } else {
                clone_repository(url, &branch, &dest);
                // Rollback: if clone failed, revert the addition to nidus.json
                if !Path::new(&dest).exists() {
                    eprintln!("{} Clone failed — reverting nidus.json...", "⚠️".yellow());
                    if let Err(e) = global_dto.remove_dependency(url) {
                        eprintln!("{} Could not revert: {}", "⚠️".yellow(), e);
                    }
                }
            }

            println!("{}", "\n🔗 Updating .dproj search paths...".cyan());
            if let Err(e) = dproj::update_all_dprojs_in_cwd(&path_src) {
                eprintln!("{} {}", "⚠️  Could not update .dproj:".yellow(), e);
            }
            return;
        }

        // ── Regular install: clone all dependencies in nidus.json ─────────────
        // If nidus.json does not exist, create a default one and reload the config.
        if !Path::new("nidus.json").exists() {
            match std::fs::write("nidus.json", DEFAULT_NIDUS_JSON) {
                Ok(_) => {
                    println!("{}", "📄 nidus.json not found — created with default Nidus dependencies.".cyan());
                    println!("{}", "   Edit nidus.json to customize your dependencies, then re-run `Nidus install`.".dimmed());
                    println!();
                }
                Err(e) => {
                    eprintln!("{} Could not create nidus.json: {}", "❌".red(), e);
                    return;
                }
            }
            if let Err(e) = global_dto.reload() {
                utils::handle_error(e);
            }
        }

        // Resolve the main source directory from the global DTO
        let path_src: String = match global_dto.get_command_install() {
            Some(cmd) => cmd.mainsrc.clone(),
            None => {
                eprintln!("{}", "❌ Could not load nidus.json configuration.".red());
                return;
            }
        };

        // Proceed only if install configuration is present
        if let Some(command_install) = global_dto.get_command_install() {
            let mut total = 0;
            let mut skipped = 0;

            println!("{}", "\n🔧 Starting dependency installation...\n".cyan());

            // Separate already-existing from to-clone
            let mut to_clone: Vec<(String, String, String)> = Vec::new();

            for (repo_url, version) in &command_install.dependencies {
                total += 1;
                let repo_name: String = utils::extract_repo_name(repo_url).unwrap();
                let destination_folder: String = format!("{}/{}", path_src, repo_name);

                if Path::new(&destination_folder).exists() {
                    println!("🔁 {} already exists. Skipping...", repo_name.blue());
                    skipped += 1;
                } else {
                    to_clone.push((repo_url.clone(), version.clone(), destination_folder));
                }
            }

            // Clone in parallel when there are multiple repositories to clone
            let results: Arc<Mutex<Vec<(String, bool)>>> = Arc::new(Mutex::new(Vec::new()));
            let mut handles = vec![];

            for (repo_url, version, dest) in to_clone {
                let url = repo_url.clone();
                let branch = version.clone();
                let results = Arc::clone(&results);

                handles.push(thread::spawn(move || {
                    println!("  🔄 Cloning {}...", url);
                    let ok = clone_repository_quiet(&url, &branch, &dest);
                    if ok {
                        println!("  {} {}", "✅ Cloned:".green(), url.green());
                    } else {
                        println!("  {} {}", "❌ Failed:".red(), url.red());
                    }
                    results.lock().unwrap().push((url, ok));
                }));
            }

            for h in handles {
                let _ = h.join();
            }

            let results_guard = results.lock().unwrap();
            let success = results_guard.iter().filter(|(_, ok)| *ok).count();
            let failed = results_guard.iter().filter(|(_, ok)| !ok).count();

            // Final summary
            println!("\n{}", "🎯 Installation summary".bold().cyan());
            println!(
                "{}: {}",
                "📦 Total dependencies".bold(),
                total.to_string().yellow()
            );
            println!(
                "{}: {}",
                "✅ Cloned successfully".bold(),
                success.to_string().green()
            );
            println!(
                "{}: {}",
                "🔁 Already existed".bold(),
                skipped.to_string().blue()
            );
            println!(
                "{}: {}",
                "❌ Failed to clone".bold(),
                failed.to_string().red()
            );

            // Write nidus.lock with the committed SHA of each dependency
            let all_deps: std::collections::HashMap<String, String> = command_install
                .dependencies
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            lockfile::write_lock(path_src.trim_end_matches('/'), &all_deps);

            // Automatically sync .dproj search paths after installation
            println!("{}", "\n🔗 Updating .dproj search paths...".cyan());
            if let Err(e) = dproj::update_all_dprojs_in_cwd(&path_src) {
                eprintln!("{} {}", "⚠️  Could not update .dproj:".yellow(), e);
                // warning only — .dproj may not exist yet (created later in Delphi IDE)
            }
        } else {
            println!("{}", "❌ No initialization command found.".red());
        }
    }
}

/// Clones a single repository to `dest`, showing a progress bar.
/// Prints success or failure message. Used by both regular install and `--add`.
fn clone_repository(repo_url: &str, branch: &str, dest: &str) {
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username, allowed| {
        if allowed.is_ssh_key() {
            git2::Cred::ssh_key_from_agent(username.unwrap_or("git"))
        } else {
            git2::Cred::default()
        }
    });
    callbacks.transfer_progress(|stats: Progress| {
        if stats.received_objects() == stats.total_objects() {
            print!(
                "\r\x1b[K{} {}/{} objects ({:.1}%)",
                "Resolving deltas:".cyan(),
                stats.indexed_deltas(),
                stats.total_deltas(),
                if stats.total_deltas() > 0 {
                    stats.indexed_deltas() as f32 / stats.total_deltas() as f32 * 100.0
                } else {
                    100.0
                }
            );
        } else if stats.total_objects() > 0 {
            let percent = stats.received_objects() as f32 / stats.total_objects() as f32;
            let filled = (percent * 30.0).round() as usize;
            let bar = format!("[{}{}]", "█".repeat(filled), "─".repeat(30 - filled));
            print!("\r\x1b[K{} {} {:.1}%", "Receiving:".cyan(), bar, percent * 100.0);
        }
        std::io::stdout().flush().unwrap();
        true
    });

    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    let mut builder = RepoBuilder::new();
    builder.fetch_options(fetch_options);
    if !branch.is_empty() {
        builder.branch(branch);
    }

    match builder.clone(repo_url, Path::new(dest)) {
        Ok(_) => {
            println!();
            println!("{} {}", "✅ Cloned:".green(), repo_url.green());
        }
        Err(e) => {
            println!();
            println!("{} {}: {}", "❌ Failed cloning".red(), repo_url.red(), e.to_string().red());
        }
    }
}

/// Clones a single repository to `dest` without progress callbacks (suitable for parallel use).
/// Returns `true` on success, `false` on failure.
pub(crate) fn clone_repository_quiet(repo_url: &str, branch: &str, dest: &str) -> bool {
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username, allowed| {
        if allowed.is_ssh_key() {
            git2::Cred::ssh_key_from_agent(username.unwrap_or("git"))
        } else {
            git2::Cred::default()
        }
    });

    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    let mut builder = RepoBuilder::new();
    builder.fetch_options(fetch_options);
    if !branch.is_empty() {
        builder.branch(branch);
    }

    builder.clone(repo_url, Path::new(dest)).is_ok()
}

/// Validates cloned repositories against nidus.lock.
/// Exits with a non-zero code if the lock is missing or any repo diverges.
fn frozen_check(global_dto: &mut ConfigGlobalDTO) {
    use crate::core::core_lockfile::lockfile;

    let lock = match crate::dto::lock_dto::NidusLock::load() {
        Some(l) => l,
        None => {
            eprintln!(
                "{} nidus.lock not found. Run `Nidus install` first to generate it.",
                "❌".red()
            );
            std::process::exit(1);
        }
    };

    let install = match global_dto.get_command_install() {
        Some(c) => c,
        None => {
            eprintln!("{} nidus.json not loaded.", "❌".red());
            std::process::exit(1);
        }
    };

    let mainsrc = install.mainsrc.trim_end_matches('/').to_string();
    let mut mismatches = 0usize;

    println!("{}", "\n🔒 Frozen install — validating against nidus.lock\n".bold().cyan());

    for url in install.dependencies.keys() {
        let name = match utils::extract_repo_name(url) {
            Some(n) => n,
            None => continue,
        };
        let dest = format!("{}/{}", mainsrc, name);

        let locked_entry = match lock.dependencies.get(url) {
            Some(e) => e,
            None => {
                eprintln!(
                    "  {} {} — not in nidus.lock (run `Nidus install` to update the lock)",
                    "❌".red(),
                    name.bold()
                );
                mismatches += 1;
                continue;
            }
        };

        let actual_sha = lockfile::read_commit_sha(&dest);

        match actual_sha {
            None => {
                eprintln!("  {} {} — not cloned", "❌".red(), name.bold());
                mismatches += 1;
            }
            Some(sha) if sha != locked_entry.commit => {
                eprintln!(
                    "  {} {} — commit mismatch (lock: {}, actual: {})",
                    "❌".red(),
                    name.bold(),
                    &locked_entry.commit[..7],
                    &sha[..7],
                );
                mismatches += 1;
            }
            Some(_) => {
                println!("  {} {}", "✅".green(), name.bold());
            }
        }
    }

    println!();
    if mismatches > 0 {
        eprintln!(
            "{}",
            format!("❌ {} repo(s) do not match nidus.lock — aborting.", mismatches)
                .bold()
                .red()
        );
        std::process::exit(1);
    } else {
        println!("{}", "✅ All dependencies match nidus.lock.\n".bold().green());
    }

}
