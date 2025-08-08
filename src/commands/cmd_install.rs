use super::super::core::core_utils::utils;
use super::super::dto::config_global_dto::ConfigGlobalDTO;
use super::command_trait::cmd_trait::ICommand;
use clap::{Arg, Command};
use colored::*;
use git2::{build::RepoBuilder, FetchOptions, Progress, RemoteCallbacks};
use std::io::Write;
use std::path::Path;

pub struct CommandInstall;

impl ICommand for CommandInstall {
    fn new() -> Self {
        Self
    }
    fn arg() -> Arg {
        Arg::new("")
    }

    fn command() -> Command {
        Command::new("install").about("📦 Install the nest4d framework and its dependencies.")
    }

    fn execute(global_dto: &mut ConfigGlobalDTO, _matches: &clap::ArgMatches) {
        // Obtém o diretório principal do código-fonte do DTO global
        let path_src: String = global_dto
            .get_command_install()
            .unwrap()
            .mainsrc
            .to_string();

        // Verifica se há um comando de inicialização definido no DTO global
        if let Some(command_install) = global_dto.get_command_install() {
            let mut total = 0;
            let mut skipped = 0;
            let mut success = 0;
            let mut failed = 0;

            println!("{}", "\n🔧 Starting dependency installation...\n".cyan());

            // Itera sobre as dependências definidas no comando de inicialização
            for (repo_url, version) in &command_install.dependencies {
                total += 1;

                // Extrai o nome do repositório a partir da URL
                let repo_name: String = utils::extract_repo_name(repo_url).unwrap();
                // Define o diretório de destino para clonar o repositório
                let destination_folder: String = format!("{}/{}", path_src, repo_name);

                // Se já existir, pula a clonagem
                if Path::new(&destination_folder).exists() {
                    println!("🔁 {} already exists. Skipping...", repo_name.blue());
                    skipped += 1;
                    continue;
                }

                // Monta o comando de clonagem (exibido apenas para referência visual)
                let full_command: String = format!(
                    "git clone {} -b {} {}",
                    repo_url, version, destination_folder
                );

                // Configura callbacks para progresso da transferência
                let mut callbacks: RemoteCallbacks = RemoteCallbacks::new();
                callbacks.credentials(|_, _, _| git2::Cred::ssh_key_from_agent("git"));
                callbacks.transfer_progress(|stats: Progress| {
                    if stats.received_objects() == stats.total_objects() {
                        print!(
                            "\r\x1b[K{} {}/{} objects ({:.1}%)",
                            "Resolving deltas:".cyan(),
                            stats.indexed_deltas(),
                            stats.total_deltas(),
                            stats.indexed_deltas() as f32 / stats.total_deltas() as f32 * 100.0
                        );
                    } else if stats.total_objects() > 0 {
                        let percent: f32 =
                            stats.received_objects() as f32 / stats.total_objects() as f32;
                        let bar_length: usize = 30;
                        let filled: usize = (percent * bar_length as f32).round() as usize;

                        let bar: String = format!(
                            "[{}{}]",
                            "█".repeat(filled),
                            "─".repeat(bar_length - filled)
                        );

                        print!(
                            "\r\x1b[K{} {} {:.1}%",
                            "Receiving dependency:".cyan(),
                            bar,
                            percent * 100.0
                        );
                    }
                    std::io::stdout().flush().unwrap();
                    true
                });

                // Configura opções de fetch
                let mut fetch_options: FetchOptions = FetchOptions::new();
                fetch_options.remote_callbacks(callbacks);

                // Constrói o repositório
                let mut builder: RepoBuilder = RepoBuilder::new();
                builder.fetch_options(fetch_options);

                // Clona o repositório
                match builder.clone(repo_url, Path::new(&destination_folder)) {
                    Ok(_) => {
                        println!();
                        println!(
                            "{} {}",
                            "✅ Repository cloned successfully:".green(),
                            full_command.green()
                        );
                        success += 1;
                    }
                    Err(e) => {
                        println!();
                        println!(
                            "{} {}: {}",
                            "❌ Failed cloning into".red(),
                            repo_name.red(),
                            e.to_string().red()
                        );
                        failed += 1;
                    }
                }
            }

            // 🧾 Resumo final
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
        } else {
            println!("{}", "❌ No initialization command found.".red());
        }
    }
}
