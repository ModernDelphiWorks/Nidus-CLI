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

#[cfg(test)]
mod tests {
    use super::project_unit::add_units_to_dpr;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Creates a minimal .dpr with a `uses` block and returns (TempDir, .dpr path).
    fn make_dpr(content: &str) -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("MyProject.dpr");
        std::fs::write(&path, content).unwrap();
        (dir, path)
    }

    fn dpr_template() -> &'static str {
        "program MyProject;\n\nuses\n  SysUtils,\n  AppModule in 'src\\AppModule.pas';\n\nbegin\nend.\n"
    }

    // ─── 1. Add a new unit ─────────────────────────────────────────────────

    #[test]
    fn test_adds_single_unit_to_dpr() {
        let (_dir, dpr_path) = make_dpr(dpr_template());
        let unit_path = dpr_path.parent().unwrap().join("src/modules/user/UserModule.pas");

        add_units_to_dpr(&dpr_path, &[("UserModule".to_string(), unit_path)]).unwrap();

        let content = std::fs::read_to_string(&dpr_path).unwrap();
        assert!(content.contains("UserModule in '"), "unit declaration must be present");
    }

    // ─── 2. Add multiple units at once ─────────────────────────────────────

    #[test]
    fn test_adds_multiple_units_at_once() {
        let (_dir, dpr_path) = make_dpr(dpr_template());
        let base = dpr_path.parent().unwrap().to_path_buf();
        let units = vec![
            ("UserModule".to_string(),     base.join("src/modules/user/UserModule.pas")),
            ("UserController".to_string(), base.join("src/modules/user/UserController.pas")),
            ("UserService".to_string(),    base.join("src/modules/user/UserService.pas")),
        ];

        add_units_to_dpr(&dpr_path, &units).unwrap();

        let content = std::fs::read_to_string(&dpr_path).unwrap();
        assert!(content.contains("UserModule in '"));
        assert!(content.contains("UserController in '"));
        assert!(content.contains("UserService in '"));
    }

    // ─── 3. Does not duplicate a unit already present in .dpr ───────────────

    #[test]
    fn test_does_not_duplicate_existing_unit() {
        let (_dir, dpr_path) = make_dpr(dpr_template());
        let unit_path = dpr_path.parent().unwrap().join("src/modules/user/UserModule.pas");

        // First insertion
        add_units_to_dpr(&dpr_path, &[("UserModule".to_string(), unit_path.clone())]).unwrap();
        // Second insertion — must be ignored
        add_units_to_dpr(&dpr_path, &[("UserModule".to_string(), unit_path)]).unwrap();

        let content = std::fs::read_to_string(&dpr_path).unwrap();
        assert_eq!(
            content.matches("UserModule in '").count(),
            1,
            "unit declaration must appear exactly once"
        );
    }

    // ─── 4. Empty units list leaves the file unchanged ──────────────────────

    #[test]
    fn test_empty_units_list_leaves_file_unchanged() {
        let (_dir, dpr_path) = make_dpr(dpr_template());
        let original = std::fs::read_to_string(&dpr_path).unwrap();

        add_units_to_dpr(&dpr_path, &[]).unwrap();

        let after = std::fs::read_to_string(&dpr_path).unwrap();
        assert_eq!(original, after, "file must be unchanged when units list is empty");
    }

    // ─── 5. File without `uses` block — content unchanged ──────────────────

    #[test]
    fn test_no_uses_block_leaves_file_unchanged() {
        let content = "program MyProject;\n\nbegin\nend.\n";
        let (_dir, dpr_path) = make_dpr(content);
        let unit_path = dpr_path.parent().unwrap().join("src/modules/user/UserModule.pas");

        add_units_to_dpr(&dpr_path, &[("UserModule".to_string(), unit_path)]).unwrap();

        let after = std::fs::read_to_string(&dpr_path).unwrap();
        assert_eq!(content, after, "file without `uses` must not be modified");
    }

    // ─── 6. `uses` without semicolon — content unchanged ───────────────────

    #[test]
    fn test_uses_without_semicolon_leaves_file_unchanged() {
        let content = "program MyProject;\n\nuses\n  SysUtils\n\nbegin\nend.\n";
        let (_dir, dpr_path) = make_dpr(content);
        let unit_path = dpr_path.parent().unwrap().join("src/UserModule.pas");

        add_units_to_dpr(&dpr_path, &[("UserModule".to_string(), unit_path)]).unwrap();

        let after = std::fs::read_to_string(&dpr_path).unwrap();
        assert_eq!(content, after, "file with unterminated `uses` must not be modified");
    }

    // ─── 7. Relative path is written correctly into .dpr ────────────────────

    #[test]
    fn test_relative_path_written_into_dpr() {
        let (_dir, dpr_path) = make_dpr(dpr_template());
        // Use absolute path; pathdiff must compute the relative one
        let unit_abs = dpr_path.parent().unwrap().join("src/modules/order/OrderService.pas");
        // Ensure the file exists so pathdiff works with a real absolute path
        std::fs::create_dir_all(unit_abs.parent().unwrap()).unwrap();
        std::fs::write(&unit_abs, "").unwrap();

        add_units_to_dpr(&dpr_path, &[("OrderService".to_string(), unit_abs)]).unwrap();

        let content = std::fs::read_to_string(&dpr_path).unwrap();
        // Relative path must not contain the absolute TempDir prefix
        let line = content
            .lines()
            .find(|l| l.contains("OrderService in '"))
            .expect("OrderService declaration must be in .dpr");
        assert!(
            !line.contains("/tmp") && !line.contains("\\\\"),
            "path in .dpr should be relative, got: {}",
            line
        );
    }

    // ─── 8. Insertion preserves pre-existing units ──────────────────────────

    #[test]
    fn test_pre_existing_units_are_preserved() {
        let (_dir, dpr_path) = make_dpr(dpr_template());
        let unit_path = dpr_path.parent().unwrap().join("src/modules/user/UserModule.pas");

        add_units_to_dpr(&dpr_path, &[("UserModule".to_string(), unit_path)]).unwrap();

        let content = std::fs::read_to_string(&dpr_path).unwrap();
        // Original template units must remain present
        assert!(content.contains("SysUtils"), "SysUtils must be preserved");
        assert!(content.contains("AppModule in '"), "AppModule must be preserved");
    }

    // ─── 9. Successive calls accumulate units ───────────────────────────────

    #[test]
    fn test_successive_calls_accumulate_units() {
        let (_dir, dpr_path) = make_dpr(dpr_template());
        let base = dpr_path.parent().unwrap().to_path_buf();

        add_units_to_dpr(
            &dpr_path,
            &[("UserModule".to_string(), base.join("src/modules/user/UserModule.pas"))],
        ).unwrap();
        add_units_to_dpr(
            &dpr_path,
            &[("ProductModule".to_string(), base.join("src/modules/product/ProductModule.pas"))],
        ).unwrap();

        let content = std::fs::read_to_string(&dpr_path).unwrap();
        assert!(content.contains("UserModule in '"));
        assert!(content.contains("ProductModule in '"));
    }
}
