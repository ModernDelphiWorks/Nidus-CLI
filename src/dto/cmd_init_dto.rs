use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandInitDTO {
    pub name: String,
    pub description: String,
    pub version: String,
    pub homepage: String,
    pub mainsrc: String,
    pub projects: Vec<String>,
    pub dependencies: HashMap<String, String>,
}
