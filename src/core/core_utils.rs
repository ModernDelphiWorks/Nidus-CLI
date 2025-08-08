pub mod utils {
    use crate::error::{CliError, Result};
    use colored::*;
    use log::{debug, info, warn};
    use regex::Regex;
    use std::fs::{self, File};
    use std::io::{self, Read, Write};
    use std::path::{Path, PathBuf};

    pub fn version() -> String {
        version_str().to_string()
    }

    pub fn version_str() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    pub fn get_size_file(path: &str) -> Result<String> {
        debug!("Calculando tamanho do arquivo: {}", path);
        let mut file: File = File::open(path)?;
        let mut buffer: Vec<u8> = Vec::new();
        file.read_to_end(&mut buffer)?;

        let size: usize = buffer.len();
        Ok(format!("({} bytes)", size))
    }

    pub fn read_from_file(file_path: &str) -> Result<String> {
        debug!("Lendo arquivo: {}", file_path);
        let mut file: File = File::open(file_path)?;
        let mut content: String = String::new();
        file.read_to_string(&mut content)?;

        Ok(content)
    }

    pub fn write_to_file(file_path: &str, content: &str) -> Result<()> {
        debug!("Escrevendo arquivo: {}", file_path);
        let mut file: File = File::create(file_path)?;
        file.write_all(content.as_bytes())?;
        info!("✅ Arquivo criado: {}", file_path);

        Ok(())
    }

    pub fn regex_replace_all(input: &str, pattern: &str, replacement: &str) -> Result<String> {
        debug!("Aplicando regex: {} -> {}", pattern, replacement);
        let regex_pattern: Regex = Regex::new(pattern)?;
        Ok(regex_pattern.replace_all(input, replacement).to_string())
    }

    // Extract git name the url
    pub fn extract_repo_name(url: &str) -> Option<String> {
        let parts: Vec<&str> = url.split('/').collect();
        let last_part: &&str = parts.last()?;

        let name_end: usize = last_part.rfind('.')?;
        Some(last_part[..name_end].to_string())
    }

    pub fn write_file_with_stats(path: &Path, content: &str) -> io::Result<(PathBuf, u64, String)> {
        write_to_file(path.to_string_lossy().as_ref(), content)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        let bytes = std::fs::metadata(path)?.len();
        let size_str = format!("({} bytes)", bytes);

        Ok((
            PathBuf::from(path.to_string_lossy().replace("\\", "/")),
            bytes,
            size_str,
        ))
    }

    // Check command init executed
    pub fn check_init_json_exist(_matches: &clap::ArgMatches) {
        let path = Path::new("nest4d.json");
        if !path.exists() {
            warn!("Arquivo nest4d.json não encontrado, gerando configuração padrão");
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
  "download": "https://github.com/ModernDelphiWorks/nest4d.git",
  "dependencies": {
    "https://github.com/HashLoad/Horse.git": "",
    "https://github.com/ModernDelphiWorks/evolution4d.git": "",
    "https://github.com/ModernDelphiWorks/injector4d.git": ""
  }
}"#;
            if let Err(e) = fs::write(path, default_json) {
                eprintln!("❌ Falha ao criar nest4d.json: {}", e);
                std::process::exit(1);
            }
            info!("Arquivo nest4d.json criado com sucesso");
            println!("{}", "✅ Default nest4d.json created.".green());
        }
    }

    /// Imprime mensagens de erro e encerra o processo
    pub fn println_panic(messages: &[&str]) {
        for msg in messages {
            eprintln!("{}", msg);
        }
        std::process::exit(1);
    }
    
    /// Versão que aceita CliError
    pub fn handle_error(error: CliError) -> ! {
        eprintln!("❌ {}", error);
        std::process::exit(1);
    }

    pub fn path_to_unix_style(path: &Path) -> String {
        path.display().to_string().replace("\\", "/")
    }

    pub fn camel_case(s: &str) -> String {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }
}
