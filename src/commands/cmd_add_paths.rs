use super::super::dto::config_global_dto::ConfigGlobalDTO;
use super::command_trait::cmd_trait::CliCommand;
use crate::core::core_add_paths_dproj::dproj;
use crate::core::core_utils::utils;
use crate::error::CliError;
use clap::{Arg, Command};

pub struct CommandAddPaths;

impl CliCommand for CommandAddPaths {

    fn arg() -> Arg {
        Arg::new("")
    }

    fn command() -> Command {
        Command::new("sync")
            .about("🔗 Add dependencies' source paths to .dproj file")
            .visible_alias("add-paths")
    }

    fn execute(_global_dto: &mut ConfigGlobalDTO, _matches: &clap::ArgMatches) {
        let mainsrc = _global_dto
            .get_command_install()
            .map(|c| c.mainsrc.clone())
            .unwrap_or_else(|| "./dependencies".to_string());

        let (dproj_files, dependency_paths) =
            dproj::find_dproj_and_collect_paths(&mainsrc).unwrap_or_default();

        if dproj_files.is_empty() {
            utils::handle_error(CliError::validation_error(
                "No .dproj file found. Open Delphi, save the project, then re-run from the project root."
            ));
        }

        if dependency_paths.is_empty() {
            utils::handle_error(CliError::validation_error(format!(
                "No `src`/`Source` folders found under {}",
                mainsrc
            )));
        }

        let deps_ref: Vec<&str> = dependency_paths.iter().map(|s| s.as_str()).collect();

        let mut had_error = false;
        for dproj_path in dproj_files {
            match dproj::add_search_paths_to_dproj(&dproj_path, &deps_ref) {
                Ok(_) => println!("✅ Paths added successfully to {}", dproj_path),
                Err(err) => {
                    eprintln!("❌ Failed to update .dproj: {}", dproj_path);
                    eprintln!("↳ {}", err);
                    had_error = true;
                }
            }
        }

        if had_error {
            utils::handle_error(CliError::validation_error("One or more .dproj files could not be updated."));
        }
    }
}
