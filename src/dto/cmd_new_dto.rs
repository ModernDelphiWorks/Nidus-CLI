use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub struct CommandNewDTO {
    path: PathBuf,
    project_name: String,
}

impl CommandNewDTO {
    pub fn new(path: PathBuf, project_name: String) -> Self {
        Self { path, project_name }
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }
}

impl fmt::Display for CommandNewDTO {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Path: {:?}, Project: {}", self.path, self.project_name)
    }
}
