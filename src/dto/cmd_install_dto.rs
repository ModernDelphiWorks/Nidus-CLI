use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandInstallDTO {
    pub name: String,
    pub description: String,
    pub version: String,
    pub homepage: String,
    pub mainsrc: String,
    pub projects: Vec<String>,
    pub download: String,
    pub dependencies: HashMap<String, String>,
}
