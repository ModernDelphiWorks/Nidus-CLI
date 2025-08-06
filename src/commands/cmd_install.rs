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
        Command::new("install").about("Install the nest4d framework and its dependencies.")
    }

    fn execute(_global_dto: &mut ConfigGlobalDTO, _matches: &clap::ArgMatches) {
        // Obtém o diretório principal do código-fonte do DTO global
        let path_src: String = _global_dto.get_command_init().unwrap().mainsrc.to_string();

        // Verifica se há um comando de inicialização definido no DTO global
        if let Some(command_init) = _global_dto.get_command_init() {
            // Itera sobre as dependências definidas no comando de inicialização
            for (repo_url, version) in &command_init.dependencies {
                // Extrai o nome do repositório a partir da URL
                let repo_name: String = utils::extract_repo_name(repo_url).unwrap();
                // Define o diretório de destino para clonar o repositório
                let destination_folder: String = format!("{}/{}", path_src, repo_name);

                // Monta o comando completo para clonar o repositório
                let full_command: String = format!(
                    "git clone {} -b {} {}",
                    repo_url, version, destination_folder
                );

                // Configura os callbacks para capturar o progresso da transferência
                let mut callbacks: RemoteCallbacks = RemoteCallbacks::new();
                callbacks.credentials(|_, _, _| git2::Cred::ssh_key_from_agent("git"));
                callbacks.transfer_progress(|stats: Progress| {
                    // Exibe o progresso da transferência na saída padrão
                    if stats.received_objects() == stats.total_objects() {
                        print!(
                            "\r\x1b[K{} {}/{} objects ({:.1}%)",
                            "Resolving deltas:".cyan(),
                            stats.indexed_deltas(),
                            stats.total_deltas(),
                            stats.indexed_deltas() as f32 / stats.total_deltas() as f32 * 100.0
                        );
                    } else if stats.total_objects() > 0 {
                        print!(
                            "\r\x1b[K{} {}/{} objects ({:.1}%)",
                            "Receiving objects:".cyan(),
                            stats.received_objects(),
                            stats.total_objects(),
                            stats.received_objects() as f32 / stats.total_objects() as f32 * 100.0
                        );
                    }
                    std::io::stdout().flush().unwrap();
                    true
                });

                // Configura as opções de fetch para incluir os callbacks
                let mut fetch_options: FetchOptions = FetchOptions::new();
                fetch_options.remote_callbacks(callbacks);

                // Constrói o repositório com as opções de fetch configuradas
                let mut builder: RepoBuilder = RepoBuilder::new();
                builder.fetch_options(fetch_options);

                // Tenta clonar o repositório utilizando o builder configurado
                match builder.clone(repo_url, Path::new(&destination_folder)) {
                    // Em caso de sucesso, exibe uma mensagem indicando o sucesso da operação
                    Ok(_) => {
                        println!();
                        println!(
                            "{} {}",
                            "✅ Repository cloned successfully:".green(),
                            full_command.green()
                        );
                    }
                    // Em caso de erro, exibe uma mensagem indicando a falha na operação
                    Err(e) => println!(
                        "{}: {}",
                        "❌ Failed cloning into:".red(),
                        e.to_string().red()
                    ),
                }
            }
        } else {
            // Se nenhum comando de inicialização for encontrado, exibe uma mensagem de erro
            println!("{}", "❌ No initialization command found.".red());
        }
    }
}
