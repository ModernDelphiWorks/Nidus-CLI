pub mod project {
    use std::{fs, io, path::PathBuf};

    use crate::core::core_utils::utils;
    use crate::templates::template_manager::TemplateManager;
    use crate::error::CliError;

    /// Resolves project template content intelligently (disk-first, then built-in fallback)
    fn get_project_template_content(template_manager: &mut TemplateManager, template_name: &str) -> Result<String, CliError> {
        if let Some(content) = utils::resolve_custom_template(template_manager, &format!("project-{}", template_name), template_name) {
            return Ok(content);
        }
        let builtin = match template_name {
            "project"    => include_str!("../templates/project/project.dpr"),
            "app_module" => include_str!("../templates/project/app_module.pas"),
            _ => return Err(CliError::ValidationError(format!("Unknown project template: {}", template_name))),
        };
        Ok(builtin.to_string())
    }

    /// Generates the initial structure of a Nidus project.
    ///
    /// # Parameters
    /// - `project_path`: base path where the project directory will be created
    /// - `project_name`: project name (also the directory name)
    /// - `include_tests`: when `true`, also creates a `test/` directory
    pub fn generate_project_structure(
        project_path: PathBuf,
        project_name: &str,
        include_tests: bool,
    ) -> io::Result<()> {
        use colored::*;

        let root = project_path.join(project_name);
        let src_path = root.join("src");
        let modules_path = src_path.join("modules");

        fs::create_dir_all(&modules_path)?;

        let mut created_files: Vec<(PathBuf, u64, String)> = Vec::new();
        let mut created_dirs: Vec<PathBuf> = Vec::new();

        // Initialize the TemplateManager
        let templates_dir = utils::get_templates_directory().map_err(|e| {
            io::Error::other(format!("Template directory error: {}", e))
        })?;
        let mut template_manager = TemplateManager::new(templates_dir).map_err(|e| {
            io::Error::other(format!("TemplateManager error: {}", e))
        })?;

        // DPR — resolve template (disk-first, then built-in)
        let dpr_template = get_project_template_content(&mut template_manager, "project").map_err(|e| {
            io::Error::other(format!("Template error: {}", e))
        })?;
        let dpr_content = dpr_template.replace("{{project}}", project_name);
        created_files.push(utils::write_file_with_stats(
            &root.join(format!("{}.dpr", project_name)),
            &dpr_content,
        )?);

        // AppModule — resolve template (disk-first, then built-in)
        let app_module_template = get_project_template_content(&mut template_manager, "app_module").map_err(|e| {
            io::Error::other(format!("Template error: {}", e))
        })?;
        created_files.push(utils::write_file_with_stats(
            &src_path.join("AppModule.pas"),
            &app_module_template,
        )?);

        // .gitignore for Delphi projects
        let gitignore_content = "\
# Delphi compiled output\n\
*.exe\n\
*.dcu\n\
*.dcp\n\
*.bpl\n\
*.bpi\n\
*.drc\n\
*.map\n\
*.dsk\n\
*.local\n\
*.identcache\n\
*.tvsconfig\n\
\n\
# Build output\n\
Win32/\n\
Win64/\n\
OSX32/\n\
OSX64/\n\
Android/\n\
iOSDevice32/\n\
iOSDevice64/\n\
iOSSimulator/\n\
\n\
# Delphi IDE history\n\
__history/\n\
__recovery/\n\
\n\
# Project-local settings\n\
*.dproj.local\n\
*.groupproj.local\n\
\n\
# Backup files\n\
*.~*\n\
\n\
# Logs\n\
*.log\n\
";
        created_files.push(utils::write_file_with_stats(
            &root.join(".gitignore"),
            gitignore_content,
        )?);

        // Optional test folder
        if include_tests {
            let test_dir = root.join("test");
            fs::create_dir_all(&test_dir)?;
            created_dirs.push(test_dir);
        }

        // Summary
        println!("\n{}", "🎯 Project scaffold summary".bold().cyan());
        println!("📁 Root: {}", root.display());

        let mut total_bytes: u64 = 0;
        for (path, bytes, size_str) in &created_files {
            println!(
                "  • {} {}",
                utils::path_to_unix_style(path),
                size_str.yellow()
            );

            total_bytes += *bytes;
        }
        if !created_dirs.is_empty() {
            println!("{}", "📂 Directories created:".bold());
            for dir in &created_dirs {
                println!("  • {}", utils::path_to_unix_style(dir));
            }
        }
        println!(
            "{} {} ({} {})",
            "✅ Files created:".bold(),
            created_files.len(),
            "total".dimmed(),
            format!("{} bytes", total_bytes).green()
        );

        Ok(())
    }

    pub fn ensure_project_dpr_exists() -> crate::error::Result<PathBuf> {
        let dpr_files: Vec<PathBuf> = std::fs::read_dir(".")?
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                if path.extension().map(|ext| ext == "dpr").unwrap_or(false) {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        if dpr_files.is_empty() {
            return Err(crate::error::CliError::validation_error(
                "No .dpr file found. Run `Nidus new <project>` first."
            ));
        }

        // Prefer the .dpr whose stem matches the current directory name
        let cwd_name = std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()));

        let best = cwd_name
            .as_deref()
            .and_then(|name| {
                dpr_files
                    .iter()
                    .find(|p| p.file_stem().map(|s| s.to_string_lossy() == name).unwrap_or(false))
            })
            .or_else(|| dpr_files.first())
            .cloned()
            .expect("checked non-empty above");

        Ok(best)
    }
}
