use super::super::dto::config_global_dto::ConfigGlobalDTO;
use super::command_trait::cmd_trait::ICommand;
use crate::core::core_add_paths_dproj::dproj;
use clap::{Arg, Command};

pub struct CommandAddPaths;

impl ICommand for CommandAddPaths {
    fn new() -> Self {
        Self
    }

    fn arg() -> Arg {
        Arg::new("")
    }

    fn command() -> Command {
        Command::new("add-paths").about("🔗 Add dependencies' source paths to .dproj file")
    }

    fn execute(_global_dto: &mut ConfigGlobalDTO, _matches: &clap::ArgMatches) {
        let (dproj_files, dependency_paths) = match dproj::find_dproj_files_and_collect_paths() {
            Ok(tuple) => tuple,
            Err(_) => (Vec::new(), Vec::new()),
        };

        if dproj_files.is_empty() {
            eprintln!("❌ No .dproj file found in the current directory.");
            eprintln!("⚠️ Please open Delphi so that the .dproj file can be created,");
            eprintln!("   then save the project and run this command again in the project root.");
            std::process::exit(2);
        }

        if dependency_paths.is_empty() {
            eprintln!("⚠️ No `src`/`Source` folders found under ./dependencies/");
            std::process::exit(3);
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
            std::process::exit(4);
        }
    }
}
