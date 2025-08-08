use std::fmt;

#[derive(Debug, Clone)]

pub enum GenerateType {
    // Grupos compostos
    Module,   // Gera apenas module, handler
    Scaffold, // Gera controller, service, repository, interface, infra
    All,      // Gera tudo acima

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
    pub flat: bool,
    pub overwrite: bool,
}

impl CommandGenerateDTO {
    pub fn new(
        kind: GenerateType,
        name: String,
        path: Option<String>,
        flat: bool,
        overwrite: bool,
    ) -> Self {
        Self {
            kind,
            name,
            path,
            flat,
            overwrite,
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
            "Kind: {:?}, Name: {}, Path: {:?}, Flat: {}, Overwrite: {}",
            self.kind, self.name, self.path, self.flat, self.overwrite
        )
    }
}
