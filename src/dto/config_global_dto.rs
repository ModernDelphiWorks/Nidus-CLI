use super::cmd_gen_dto::CommandGenerateDTO;
use super::cmd_install_dto::CommandInstallDTO;
use super::cmd_new_dto::CommandNewDTO;
use crate::error::{CliError, Result};
use std::fmt;
use std::fs::File;
use std::io::BufReader;

pub struct ConfigGlobalDTO {
    config_json: Option<CommandInstallDTO>,
    command_new: Option<CommandNewDTO>,
    command_gen: Option<CommandGenerateDTO>,
}

impl ConfigGlobalDTO {
    pub fn new() -> Result<Self> {
        let mut global_dto = Self {
            config_json: None,
            command_new: None,
            command_gen: None,
        };
        // Silently ignore missing nidus.json — commands that require it
        // check get_command_install() themselves and return an informative error.
        let _ = global_dto.load_config_from_file();
        Ok(global_dto)
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

    /// Reloads `nidus.json` from disk, replacing the current configuration.
    pub fn reload(&mut self) -> Result<()> {
        self.load_config_from_file()
    }

    /// Adds a new dependency URL+branch to the in-memory config and persists to `nidus.json`.
    pub fn add_dependency(&mut self, url: String, branch: String) -> Result<()> {
        let config = self.config_json.as_mut()
            .ok_or_else(|| CliError::validation_error("nidus.json not loaded"))?;
        if config.dependencies.contains_key(&url) {
            return Err(CliError::validation_error(format!(
                "Dependency '{}' is already in nidus.json", url
            )));
        }
        config.dependencies.insert(url, branch);
        self.save_to_file()
    }

    /// Removes a dependency URL from the in-memory config and persists to `nidus.json`.
    pub fn remove_dependency(&mut self, url: &str) -> Result<()> {
        let config = self.config_json.as_mut()
            .ok_or_else(|| CliError::validation_error("nidus.json not loaded"))?;
        if url == config.download {
            return Err(CliError::validation_error(
                "Cannot remove the main framework dependency. Edit 'download' in nidus.json directly."
            ));
        }
        if !config.dependencies.contains_key(url) {
            return Err(CliError::validation_error(format!(
                "Dependency '{}' not found in nidus.json", url
            )));
        }
        config.dependencies.remove(url);
        self.save_to_file()
    }

    /// Serializes the current config back to `nidus.json`.
    /// The `download` URL is excluded from `dependencies` before saving
    /// because it is added in-memory only during `load_config_from_file`.
    pub fn save_to_file(&self) -> Result<()> {
        let mut config = self.config_json.as_ref()
            .ok_or_else(|| CliError::validation_error("nidus.json not loaded"))?
            .clone();
        let download_url = config.download.clone();
        config.dependencies.remove(&download_url);
        let json = serde_json::to_string_pretty(&config).map_err(CliError::JsonError)?;
        std::fs::write("nidus.json", json).map_err(CliError::IoError)?;
        Ok(())
    }

    fn load_config_from_file(&mut self) -> Result<()> {
        let file = File::open("nidus.json")
            .map_err(|_| CliError::ProjectNotFound)?;

        let reader = BufReader::new(file);

        let mut config: CommandInstallDTO = serde_json::from_reader(reader)
            .map_err(CliError::JsonError)?;

        // Validate required fields with actionable error messages
        Self::validate_config(&config)?;

        let main_repo_url = config.download.clone();
        config.dependencies.insert(main_repo_url, String::new());

        self.config_json = Some(config);
        Ok(())
    }

    fn validate_config(config: &CommandInstallDTO) -> Result<()> {
        if config.mainsrc.trim().is_empty() {
            return Err(CliError::validation_error(
                "nidus.json: field \"mainsrc\" is empty.\n   Add: \"mainsrc\": \"./dependencies\"",
            ));
        }
        if config.download.trim().is_empty() {
            return Err(CliError::validation_error(
                "nidus.json: field \"download\" is empty.\n   Add the Nidus framework URL, e.g.: \"download\": \"https://github.com/isaquepinheiro/nidus.git\"",
            ));
        }
        Ok(())
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
