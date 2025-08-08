pub mod project_unit {
    use std::path::{Path, PathBuf};

    pub fn add_units_to_dpr(dpr_path: &Path, units: &[(String, PathBuf)]) -> std::io::Result<()> {
        let mut content = std::fs::read_to_string(dpr_path)?;
        let dpr_dir = dpr_path.parent().unwrap_or(Path::new("."));

        if let Some(idx) = content.find("uses") {
            if let Some(end_idx) = content[idx..].find(';') {
                let insert_pos = idx + end_idx;

                let mut entries: Vec<String> = Vec::new();
                for (unit_name, abs_path) in units {
                    if !content.contains(unit_name) {
                        let rel_path = pathdiff::diff_paths(abs_path, dpr_dir)
                            .unwrap_or_else(|| abs_path.clone());
                        entries.push(format!(
                            "{} in '{}'",
                            unit_name,
                            rel_path.display().to_string().replace("\\\\", "\\")
                        ));
                    }
                }

                if !entries.is_empty() {
                    let units_str = format!(",\n  {}", entries.join(",\n  "));
                    content.insert_str(insert_pos, &units_str);
                }
            }
        }

        std::fs::write(dpr_path, content)?;
        Ok(())
    }
}
