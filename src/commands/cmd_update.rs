use super::super::core::core_utils::utils;
use super::super::dto::config_global_dto::ConfigGlobalDTO;
use super::command_trait::cmd_trait::ICommand;
use clap::{Arg, Command};
use colored::*;
use git2::{FetchOptions, Remote, RemoteCallbacks, Repository};

pub struct CommandUpdate;

impl ICommand for CommandUpdate {
    fn new() -> Self {
        Self
    }

    fn arg() -> Arg {
        Arg::new("")
    }

    fn command() -> Command {
        Command::new("update").about("⏳ Update the nest4d framework and its dependencies.")
    }

    fn execute(_global_dto: &mut ConfigGlobalDTO, _matches: &clap::ArgMatches) {
        let mainsrc: String = _global_dto
            .get_command_install()
            .unwrap()
            .mainsrc
            .to_string();

        if let Some(command_init) = _global_dto.get_command_install() {
            for (repo_url, version) in &command_init.dependencies {
                let repo_name: String = utils::extract_repo_name(repo_url).unwrap();
                let repo_path: String = format!("{}/{}", mainsrc, repo_name);

                let repository: Repository = match Repository::open(&repo_path) {
                    Ok(repo) => repo,
                    Err(e) => {
                        eprintln!(
                            "{} {}: {}",
                            "❌ Failed opening repository:".red(),
                            repo_path.red(),
                            e
                        );
                        continue;
                    }
                };
                let mut remote: Remote = match repository.find_remote("origin") {
                    Ok(remote) => remote,
                    Err(e) => {
                        eprintln!(
                            "{} {}: {}",
                            "❌ Failed finding remote 'origin':".red(),
                            repo_path.red(),
                            e
                        );
                        continue;
                    }
                };
                let mut callbacks: RemoteCallbacks = RemoteCallbacks::new();
                callbacks.credentials(|_, _, _| git2::Cred::ssh_key_from_agent("git"));
                let mut fetch_options: FetchOptions = FetchOptions::new();
                fetch_options.remote_callbacks(callbacks);
                let refspec: String = format!("refs/heads/{}:refs/heads/{}", version, version);
                let refspecs: [&String; 1] = [&refspec];

                match remote.fetch(&refspecs, Some(&mut fetch_options), None) {
                    Ok(_) => {
                        let full_command: String =
                            format!("git -C {} pull origin {}", repo_path, version);
                        println!("{} {}", "✅ Updated:".green(), full_command.green());
                    }
                    Err(e) => {
                        let full_command: String =
                            format!("git -C {} pull origin {}", repo_path, version);
                        eprintln!(
                            "{} {}: {}",
                            "❌ Failed updating:".red(),
                            full_command.red(),
                            e
                        );
                    }
                }
            }
        } else {
            println!("{}", "❌ No initialization command found.".red());
        }
    }
}
