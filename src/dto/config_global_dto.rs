use super::cmd_gen_dto::CommandGenerateDTO;
use super::cmd_install_dto::CommandInstallDTO;
use super::cmd_new_dto::CommandNewDTO;
use crate::core::core_utils::utils;
use colored::Colorize;
use std::fmt;
use std::fs::File;
use std::io::BufReader;

pub struct ConfigGlobalDTO {
    config_json: Option<CommandInstallDTO>,
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

    pub fn get_command_install(&self) -> Option<&CommandInstallDTO> {
        self.config_json.as_ref()
    }

    fn _load_config_from_file(&mut self) {
        let file: File = File::open("nest4d.json").unwrap_or_else(|_| {
            utils::println_panic(&[
                &"🚨 File nest4d.json not found!".red(),
                &"⚠️ File required!.".yellow(),
            ]);
            std::process::exit(1);
        });

        let reader: BufReader<File> = BufReader::new(file);

        let mut config: CommandInstallDTO = match serde_json::from_reader(reader) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("❌ Failed to parse nest4d.json: {}", e);
                std::process::exit(1);
            }
        };

        // Inclui o repositório principal (caso não esteja presente)
        let main_repo_url: String = config.download.clone();
        let main_version: String = "".to_string();
        config.dependencies.insert(main_repo_url, main_version);

        self.config_json = Some(config);
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
