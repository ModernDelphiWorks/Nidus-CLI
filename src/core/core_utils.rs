pub mod utils {
    use crate::error::{CliError, Result};
    use crate::templates::template_manager::TemplateManager;
    use log::{debug, info};
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
        debug!("Calculating file size: {}", path);
        let mut file: File = File::open(path)?;
        let mut buffer: Vec<u8> = Vec::new();
        file.read_to_end(&mut buffer)?;

        let size: usize = buffer.len();
        Ok(format!("({} bytes)", size))
    }

    pub fn read_from_file(file_path: &str) -> Result<String> {
        debug!("Reading file: {}", file_path);
        let mut file: File = File::open(file_path)?;
        let mut content: String = String::new();
        file.read_to_string(&mut content)?;

        Ok(content)
    }

    pub fn write_to_file(file_path: &str, content: &str) -> Result<()> {
        debug!("Writing file: {}", file_path);
        let mut file: File = File::create(file_path)?;
        file.write_all(content.as_bytes())?;
        info!("✅ File created: {}", file_path);

        Ok(())
    }

    pub fn regex_replace_all(input: &str, pattern: &str, replacement: &str) -> Result<String> {
        debug!("Applying regex: {} -> {}", pattern, replacement);
        let regex_pattern: Regex = Regex::new(pattern)?;
        Ok(regex_pattern.replace_all(input, replacement).to_string())
    }

    // Extract git name the url
    pub fn extract_repo_name(url: &str) -> Option<String> {
        let last_part = url.trim_end_matches('/').split('/').next_back()?;
        let name = last_part.strip_suffix(".git").unwrap_or(last_part);
        if name.is_empty() { None } else { Some(name.to_string()) }
    }

    pub fn write_file_with_stats(path: &Path, content: &str) -> io::Result<(PathBuf, u64, String)> {
        write_to_file(path.to_string_lossy().as_ref(), content)
            .map_err(|e| io::Error::other(e.to_string()))?;

        let bytes = std::fs::metadata(path)?.len();
        let size_str = format!("({} bytes)", bytes);

        Ok((
            PathBuf::from(path.to_string_lossy().replace("\\", "/")),
            bytes,
            size_str,
        ))
    }

    /// Verifies that `nidus.json` exists in the current directory.
    /// Returns an error if not found; does NOT auto-create the file.
    pub fn check_init_json_exist(_matches: &clap::ArgMatches) -> Result<()> {
        let path = Path::new("nidus.json");
        if !path.exists() {
            return Err(CliError::validation_error(
                "No nidus.json found. Run `Nidus install` to initialize this project."
            ));
        }
        Ok(())
    }

    /// Centralized exit point — the only place where process::exit is permitted
    pub fn handle_error(error: CliError) -> ! {
        eprintln!("❌ {}", error);
        std::process::exit(1);
    }

    pub fn path_to_unix_style(path: &Path) -> String {
        path.display().to_string().replace("\\", "/")
    }

    /// Returns the content of a custom (on-disk) template, or `None` if not found.
    /// Call this before falling back to the built-in template.
    pub fn resolve_custom_template(manager: &mut TemplateManager, key: &str, component: &str) -> Option<String> {
        manager.get_template(key).ok().and_then(|t| {
            t.files
                .iter()
                .find(|f| f.name.contains(component))
                .map(|f| f.content.clone())
        })
    }

    /// Returns the user templates directory (~/.Nidus/templates), creating it if needed
    pub fn get_templates_directory() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| CliError::ValidationError("Could not find home directory".to_string()))?;

        let templates_dir = PathBuf::from(home).join(".Nidus").join("templates");

        if !templates_dir.exists() {
            fs::create_dir_all(&templates_dir)?;
        }

        Ok(templates_dir)
    }

    pub fn camel_case(s: &str) -> String {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }
}
