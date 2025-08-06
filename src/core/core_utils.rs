pub mod utils {
    use colored::*;
    use regex::Regex;
    use std::error::Error;
    use std::fs::{self, File};
    use std::io::{self, Read, Write};
    use std::path::{Path, PathBuf};
    use std::result::Result;

    pub fn version() -> String {
        version_str().to_string()
    }

    pub fn version_str() -> &'static str {
        "v0.0.1"
    }

    pub fn get_size_file(path: &str) -> Result<String, Box<dyn Error>> {
        let mut file: File = File::open(path)?;
        let mut buffer: Vec<u8> = Vec::new();
        file.read_to_end(&mut buffer)?;

        let size: usize = buffer.len();
        Ok(format!("({} bytes)", size))
    }

    pub fn read_from_file(file_path: &str) -> Result<String, Box<dyn Error>> {
        let mut file: File = File::open(file_path)?;
        let mut content: String = String::new();
        file.read_to_string(&mut content)?;

        Ok(content)
    }

    pub fn write_to_file(file_path: &str, content: &str) -> Result<(), Box<dyn Error>> {
        let mut file: File = File::create(file_path)?;
        file.write_all(content.as_bytes())?;

        Ok(())
    }

    pub fn regex_replace_all(input: &str, pattern: &str, replacement: &str) -> String {
        let regex_pattern: Regex = Regex::new(pattern).unwrap();
        regex_pattern.replace_all(input, replacement).to_string()
    }

    // Extract git name the url
    pub fn extract_repo_name(url: &str) -> Option<String> {
        let parts: Vec<&str> = url.split('/').collect();
        let last_part: &&str = parts.last()?;

        let name_end: usize = last_part.rfind('.')?;
        Some(last_part[..name_end].to_string())
    }

    // Check command init executed
    pub fn check_init_json_exist(_matches: &clap::ArgMatches) {
        let path = Path::new("nest4d.json");
        if !path.exists() {
            println!(
                "{}",
                "⚠️ No nest4d.json found. Generating default config...".yellow()
            );

            let default_json = r#"{
  "name": "Nest4d",
  "description": "Nest4d Framework for Delphi",
  "version": "main",
  "homepage": "https://docs.nest4d.com/",
  "mainsrc": "./dependencies",
  "projects": [],
  "dependencies": {
    "https://github.com/HashLoad/Nest4d.git": "main"
  }
}"#;

            fs::write(path, default_json).expect("❌ Failed to write default config.");
            println!("{}", "✅ Default nest4d.json created.".green());
        }

        // let is_init_command: bool = matches.try_contains_id("init").is_ok();
        // if !is_init_command && !Path::new("nest4d.json").exists() {
        //     println_panic(&[
        //         &"🚨 File nest4d.json not found!".red(),
        //         &"⚠️ Run the 'nest4d init' command to create the configuration file.".yellow(),
        //     ]);
        // }
    }

    // Print messagens list and stop process
    pub fn println_panic(messages: &[&str]) {
        for msg in messages {
            eprintln!("{}", msg);
        }
        std::process::exit(0);
    }

    /// Gera a estrutura inicial de um projeto Nest4d.
    ///
    /// # Parâmetros
    /// - `project_path`: caminho base onde será criada a pasta do projeto
    /// - `project_name`: nome do projeto (e da pasta)
    /// - `include_tests`: se verdadeiro, cria também a pasta `test/`
    pub fn generate_project_structure(
        project_path: PathBuf,
        project_name: &str,
        include_tests: bool,
    ) -> io::Result<()> {
        let root = project_path.join(project_name);
        let src_path = root.join("src");
        let modules_path = src_path.join("modules");

        // Cria pasta do projeto
        fs::create_dir_all(&modules_path)?;

        // Cria arquivos centrais do projeto
        _create_file(
            root.join(format!("{}.dpr", project_name)),
            include_str!("../templates/project/project.dpr"),
        )?;
        _create_file(
            src_path.join("app.module.pas"),
            include_str!("../templates/project/app.module.pas"),
        )?;
        _create_file(
            src_path.join("app.server.pas"),
            include_str!("../templates/project/app.server.pas"),
        )?;

        // Cria pasta de testes se solicitado
        if include_tests {
            fs::create_dir_all(root.join("test"))?;
        }

        Ok(())
    }

    /// Gera os arquivos solicitados para um módulo Nest4d dentro de `src/modules/<module_name>/`
    ///
    /// # Parâmetros
    /// - `src_path`: caminho base da pasta `src/` do projeto
    /// - `module_name`: nome do módulo (ex: "config")
    /// - `components`: lista de componentes a gerar ("controller", "service", etc) ou "all"
    pub fn generate_module_structure(
        src_path: PathBuf,
        module_name: &str,
        components: &[&str],
    ) -> std::io::Result<()> {
        let module_dir = src_path.join("modules").join(module_name.to_lowercase());

        fs::create_dir_all(&module_dir)?;

        // Mapeia cada componente para seu respectivo template
        let all_components = [
            "module",
            "handler",
            "controller",
            "service",
            "repository",
            "interface",
            "infra",
        ];

        // Determina os componentes a serem criados
        let to_generate: Vec<&str> = if components.contains(&"all") {
            all_components.to_vec()
        } else {
            components.to_vec()
        };

        // Cria os arquivos com base nos componentes solicitados
        for comp in to_generate {
            let filename = format!("{}.{}.pas", module_name, comp);
            let filepath = module_dir.join(filename);
            let template = match comp {
                "controller" => include_str!("../templates/module/controller.pas"),
                "service" => include_str!("../templates/module/service.pas"),
                "repository" => include_str!("../templates/module/repository.pas"),
                "interface" => include_str!("../templates/module/interface.pas"),
                "infra" => include_str!("../templates/module/infra.pas"),
                "module" => include_str!("../templates/module/module.pas"),
                "handler" => include_str!("../templates/module/handler.pas"),
                _ => continue,
            };

            _create_file(filepath, template)?;
        }

        Ok(())
    }

    fn _create_file<P: AsRef<Path>>(path: P, content: &str) -> io::Result<()> {
        let mut file = fs::File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}
