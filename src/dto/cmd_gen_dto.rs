use std::fmt;

#[derive(Debug, Clone)]

pub enum GenerateType {
    // Composite groups
    Module,   // Generates only module + handler
    Scaffold, // Generates controller, service, repository, interface, infra
    All,      // Generates everything above

    // Itens individuais
    Controller,
    Service,
    Repository,
    Interface,
    Infra,
    Handler,
}

#[derive(Debug)]
pub struct CommandGenerateDTO {
    pub kind: GenerateType,
    pub name: String,
    pub path: Option<String>,
    pub overwrite: bool,
    pub template: Option<String>,
}

impl CommandGenerateDTO {
    pub fn new(
        kind: GenerateType,
        name: String,
        path: Option<String>,
        overwrite: bool,
        template: Option<String>,
    ) -> Self {
        Self {
            kind,
            name,
            path,
            overwrite,
            template,
        }
    }

    pub fn get_path(&self) -> String {
        self.path.clone().unwrap_or("./src".to_string())
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}

impl fmt::Display for CommandGenerateDTO {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Kind: {:?}, Name: {}, Path: {:?}, Overwrite: {}, Template: {:?}",
            self.kind, self.name, self.path, self.overwrite, self.template
        )
    }
}
