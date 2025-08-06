use colored::Colorize;

use crate::core::core_utils::utils;

use super::cmd_gen_dto::CommandGenerateDTO;
use super::cmd_init_dto::CommandInitDTO;
use super::cmd_new_dto::CommandNewDTO;
use std::fmt;
use std::fs::File;
use std::io::BufReader;

pub struct ConfigGlobalDTO {
    config_json: Option<CommandInitDTO>,
    command_new: Option<CommandNewDTO>,
    command_gen: Option<CommandGenerateDTO>,
}

impl ConfigGlobalDTO {
    pub fn new() -> Self {
        let mut global_dto: ConfigGlobalDTO = Self {
            config_json: None,
            command_new: None,
            command_gen: None,
        };
        global_dto._load_config_from_file();
        global_dto
    }

    pub fn set_command_new(&mut self, command_new: CommandNewDTO) {
        self.command_new = Some(command_new);
    }

    pub fn set_command_gen(&mut self, command_gen: CommandGenerateDTO) {
        self.command_gen = Some(command_gen);
    }

    pub fn get_command_new(&self) -> Option<&CommandNewDTO> {
        self.command_new.as_ref()
    }

    pub fn get_command_gen(&self) -> Option<&CommandGenerateDTO> {
        self.command_gen.as_ref()
    }

    pub fn get_command_init(&self) -> Option<&CommandInitDTO> {
        self.config_json.as_ref()
    }

    fn _load_config_from_file(&mut self) {
        match File::open("nest4d.json") {
            Ok(file) => {
                let reader = BufReader::new(file);
                match serde_json::from_reader::<_, CommandInitDTO>(reader) {
                    Ok(mut config) => {
                        let main_repo_url = "https://github.com/HashLoad/Nest4d.git";
                        let main_version = config.version.clone();
                        config
                            .dependencies
                            .insert(main_repo_url.to_string(), main_version);
                        self.config_json = Some(config);
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to parse nest4d.json: {}", e);
                    }
                }
            }
            Err(_) => {
                utils::println_panic(&[
                    &"🚨 File nest4d.json not found!".red(),
                    &"⚠️ Run the 'nest4d init' command to create the configuration file.".yellow(),
                ]);
            }
        }
    }
}

impl Default for ConfigGlobalDTO {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ConfigGlobalDTO {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "GlobalDTO {{")?;
        writeln!(f, "    config_json: {:?}", self.config_json)?;
        writeln!(f, "    command_new: {:?}", self.command_new)?;
        writeln!(f, "    command_gen: {:?}", self.command_gen)?;
        writeln!(f, "}}")
    }
}
