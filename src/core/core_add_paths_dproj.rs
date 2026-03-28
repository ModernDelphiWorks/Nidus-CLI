pub mod dproj {
    use std::{collections::HashSet, error::Error, fs, path::Path};
    use xmltree::{Element, EmitterConfig, XMLNode};
    use walkdir::{IntoIter, WalkDir};

    // =============================
    // Public Orchestration
    // =============================

    /// Finds .dproj files in CWD and dependency source paths (src/Source).
    pub fn find_dproj_files_and_collect_paths() -> Result<(Vec<String>, Vec<String>), Box<dyn Error>>
    {
        find_dproj_and_collect_paths("./dependencies")
    }

    /// Like `find_dproj_files_and_collect_paths`, but uses `mainsrc` instead of the hardcoded path.
    pub fn find_dproj_and_collect_paths(
        mainsrc: &str,
    ) -> Result<(Vec<String>, Vec<String>), Box<dyn Error>> {
        let dprojs = find_dproj_in_cwd()?;
        if dprojs.is_empty() {
            return Err("No .dproj found in current directory".into());
        }

        let dep_paths = collect_dependency_paths(mainsrc)?;
        Ok((dprojs, dep_paths))
    }

    /// Syncs all .dproj files in CWD with dependency source paths from `mainsrc`.
    /// Silently returns Ok(()) when no .dproj or no src dirs are found (non-fatal).
    pub fn update_all_dprojs_in_cwd(mainsrc: &str) -> Result<(), Box<dyn Error>> {
        let (dproj_files, dep_paths) = find_dproj_and_collect_paths(mainsrc)?;
        let deps_ref: Vec<&str> = dep_paths.iter().map(|s| s.as_str()).collect();
        for dproj_path in dproj_files {
            add_search_paths_to_dproj(&dproj_path, &deps_ref)?;
        }
        Ok(())
    }

    /// Merges additional paths into the `DCC_UnitSearchPath` node of a .dproj file.
    pub fn add_search_paths_to_dproj(
        dproj_path: &str,
        additional_paths: &[&str],
    ) -> Result<(), Box<dyn Error>> {
        const XML_TAG_PROPERTY_GROUP: &str = "PropertyGroup";
        const XML_ATTR_CONDITION: &str = "Condition";
        const XML_ATTR_CONDITION_VALUE: &str = "'$(Base)'!=''"; // Base block condition
        const XML_TAG_LIBRARY_PATH: &str = "DCC_UnitSearchPath";
        const MACRO: &str = "$(DCC_UnitSearchPath)";

        // Load and parse XML
        let content = fs::read_to_string(dproj_path)?;
        let mut root = Element::parse(content.as_bytes())?;

        // Locate the PropertyGroup with the expected Condition (DFS)
        let pg_path = find_element_path(
            &root,
            XML_TAG_PROPERTY_GROUP,
            XML_ATTR_CONDITION,
            XML_ATTR_CONDITION_VALUE,
        )
        .ok_or("PropertyGroup with expected Condition not found")?;

        let pg = get_element_at_path_mut(&mut root, &pg_path)
            .ok_or("Failed to navigate to PropertyGroup")?;

        // Find index of existing DCC_UnitSearchPath child (if any)
        let dcc_idx = pg.children.iter().position(|n| {
            matches!(n, XMLNode::Element(e) if e.name == XML_TAG_LIBRARY_PATH)
        });

        let (current_text, created_now) = if let Some(idx) = dcc_idx {
            if let XMLNode::Element(e) = &pg.children[idx] {
                (e.get_text().map(|s| s.into_owned()).unwrap_or_default(), false)
            } else {
                (String::new(), false)
            }
        } else {
            (String::new(), true)
        };

        // Build new semicolon list
        let mut items = split_semicolon_list(&current_text);
        let had_root_dot_any = items.iter().any(|s| s.trim() == r".\");

        let add_norm: Vec<String> = additional_paths
            .iter()
            .map(|p| normalize_windows_path(p))
            .collect();
        merge_semicolon_paths(&mut items, &add_norm);

        // Keep .\ as the first entry when newly created or already present
        if created_now || had_root_dot_any {
            items.retain(|s| s.trim() != r".\");
            items.insert(0, r".\".to_string());
        }

        // Ensure the macro appears exactly once at the end
        let has_macro = items.iter().any(|s| s.trim().eq_ignore_ascii_case(MACRO));
        let new_value = if !has_macro {
            items.push(MACRO.to_string());
            items.join(";")
        } else if !items
            .last()
            .map(|s| s.trim().eq_ignore_ascii_case(MACRO))
            .unwrap_or(false)
        {
            let mut without_macro: Vec<String> = items
                .into_iter()
                .filter(|s| !s.trim().eq_ignore_ascii_case(MACRO))
                .collect();
            without_macro.push(MACRO.to_string());
            without_macro.join(";")
        } else {
            items.join(";")
        };

        // Write back to the node or create it
        if let Some(idx) = dcc_idx {
            if let XMLNode::Element(e) = &mut pg.children[idx] {
                // Replace all text nodes with the new value
                e.children.retain(|n| !matches!(n, XMLNode::Text(_)));
                e.children.push(XMLNode::Text(new_value));
            }
        } else {
            let mut new_elem = Element::new(XML_TAG_LIBRARY_PATH);
            new_elem.children.push(XMLNode::Text(new_value));
            pg.children.push(XMLNode::Element(new_elem));
        }

        // Serialize back to file
        let config = EmitterConfig::new().perform_indent(true);
        let mut output = Vec::new();
        root.write_with_config(&mut output, config)?;
        fs::write(dproj_path, &output)?;

        // Post-processing: remove accidental XML header (<?xml ...?>) to preserve the Delphi first line
        fix_msbuild_header(dproj_path)?;

        Ok(())
    }

    // =============================
    // Layer 1: Project discovery
    // =============================

    /// Finds .dproj files in the current directory.
    fn find_dproj_in_cwd() -> Result<Vec<String>, Box<dyn Error>> {
        let mut out = Vec::new();
        for entry in fs::read_dir(".")?.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "dproj").unwrap_or(false) {
                out.push(path.to_string_lossy().into_owned());
            }
        }
        Ok(out)
    }

    // =============================
    // Layer 2: Dependencies scanning
    // =============================

    /// Scans `root` and returns only paths under {package}/(src|Source)/... (case-insensitive).
    fn collect_dependency_paths(root: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let mut set: HashSet<String> = HashSet::new();

        let entries: IntoIter = WalkDir::new(root).into_iter();
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            if let Ok(relative) = path.strip_prefix(root) {
                // Segments: [package, src|Source, ...]
                let parts: Vec<String> = relative
                    .iter()
                    .filter_map(|p| p.to_str())
                    .map(|s| s.to_string())
                    .collect();

                // Accept only when:
                // - there are at least 2 segments
                // - the second segment is "src" or "Source" (case-insensitive)
                if is_under_top_level_src(&parts) {
                    // Build path in the .\dependencies\{package}\(src|Source)\... format
                    let win_rel = to_win_path(relative); // e.g. "Nidus\Source\Pipes\..."
                    let trimmed_root = trim_leading_dots_slashes(root); // e.g. "dependencies"

                    // Always insert exactly ONE backslash between root and relative
                    let full = format!(r".\{}\{}", trimmed_root, win_rel.trim_start_matches('\\'));

                    // Normalize (without collapsing single backslashes)
                    set.insert(normalize_windows_path(&full));
                }
            }
        }

        Ok(set.into_iter().collect())
    }

    /// Returns true if `parts` lies under {package}/(src|Source)/...
    fn is_under_top_level_src(parts: &[String]) -> bool {
        if parts.len() < 2 {
            return false;
        }
        let second = parts[1].to_ascii_lowercase();
        second == "src" || second == "source"
    }

    /// Converts a relative `Path` to backslash notation (Windows style).
    fn to_win_path(p: &Path) -> String {
        p.to_string_lossy().replace('/', r"\").to_string()
    }

    /// Strips leading `./`, `.\\`, and slash characters from a path string.
    fn trim_leading_dots_slashes(s: &str) -> String {
        s.trim_start_matches("./")
            .trim_start_matches(".\\")
            .trim_start_matches('\\')
            .trim_start_matches('/')
            .to_string()
    }

    /// Normalizes to backslash separators, removes duplicates and trailing semicolons.
    fn normalize_windows_path(s: &str) -> String {
        let mut r = s.replace('/', r"\");
        while r.contains(r"\\") {
            r = r.replace(r"\\", r"\");
        }
        r.trim().trim_end_matches(';').to_string()
    }

    // =============================
    // Layer 3: XML helpers (.dproj)
    // =============================

    /// DFS search: returns the index path to the first element matching `tag` + attribute value.
    fn find_element_path(
        root: &Element,
        tag: &str,
        attr: &str,
        val: &str,
    ) -> Option<Vec<usize>> {
        fn walk(elem: &Element, tag: &str, attr: &str, val: &str) -> Option<Vec<usize>> {
            for (i, child) in elem.children.iter().enumerate() {
                if let XMLNode::Element(e) = child {
                    if e.name == tag
                        && e.attributes.get(attr).map(|v| v == val).unwrap_or(false)
                    {
                        return Some(vec![i]);
                    }
                    if let Some(mut sub) = walk(e, tag, attr, val) {
                        sub.insert(0, i);
                        return Some(sub);
                    }
                }
            }
            None
        }
        walk(root, tag, attr, val)
    }

    /// Navigates `root` via a slice of child indices and returns a mutable reference.
    fn get_element_at_path_mut<'a>(
        root: &'a mut Element,
        path: &[usize],
    ) -> Option<&'a mut Element> {
        if path.is_empty() {
            return Some(root);
        }
        match root.children.get_mut(path[0]) {
            Some(XMLNode::Element(child)) => get_element_at_path_mut(child, &path[1..]),
            _ => None,
        }
    }

    /// Splits a semicolon-separated list `a;b;c`, preserving order and ignoring empty entries.
    fn split_semicolon_list(s: &str) -> Vec<String> {
        s.split(';')
            .map(str::trim)
            .filter(|x| !x.is_empty())
            .map(|x| x.to_string())
            .collect()
    }

    /// Merges `more` into `dst` without duplicates, preserving the original `dst` order.
    fn merge_semicolon_paths(dst: &mut Vec<String>, more: &[String]) {
        let set: HashSet<String> = dst.iter().cloned().collect();
        for m in more {
            if !set.contains(m) && !dst.iter().any(|x| x.eq_ignore_ascii_case(m)) {
                dst.push(m.clone());
            }
        }
    }

    /// Strips the `<?xml ...?>` declaration if the XML writer inserted it unexpectedly.
    fn fix_msbuild_header(path: &str) -> Result<(), Box<dyn Error>> {
        let mut text = fs::read_to_string(path)?;
        if text.starts_with("<?xml") {
            // remove header e espaços/brancas subsequentes
            if let Some(end) = text.find("?>") {
                let after = &text[end + 2..];
                let new_text = after
                    .trim_start_matches(['\u{feff}', '\r', '\n', ' ']);
                text = new_text.to_string();
                fs::write(path, &text)?;
            }
        }
        // We do not force the first line — it is assumed to already be <Project ...> from the original file
        Ok(())
    }

    // =============================
    // Convenience wrapper
    // =============================

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::fs;
        use tempfile::TempDir;

        // ─── Fixtures ────────────────────────────────────────────────────────

        fn minimal_dproj(search_path: &str) -> String {
            format!(
                r#"<Project xmlns="http://schemas.microsoft.com/developer/msbuild/2003">
  <PropertyGroup Condition="'$(Base)'!=''">
    <DCC_UnitSearchPath>{}</DCC_UnitSearchPath>
  </PropertyGroup>
</Project>"#,
                search_path
            )
        }

        fn dproj_without_search_path() -> String {
            r#"<Project xmlns="http://schemas.microsoft.com/developer/msbuild/2003">
  <PropertyGroup Condition="'$(Base)'!=''">
  </PropertyGroup>
</Project>"#
                .to_string()
        }

        // ─── normalize_windows_path ───────────────────────────────────────

        #[test]
        fn test_normalize_windows_path_converts_slashes() {
            assert_eq!(normalize_windows_path("./dep/src"), r".\dep\src");
        }

        #[test]
        fn test_normalize_windows_path_removes_trailing_semicolon() {
            assert_eq!(normalize_windows_path(r".\dep\src;"), r".\dep\src");
        }

        #[test]
        fn test_normalize_windows_path_collapses_double_backslash() {
            assert_eq!(normalize_windows_path(r".\dep\\src"), r".\dep\src");
        }

        // ─── trim_leading_dots_slashes ────────────────────────────────────

        #[test]
        fn test_trim_leading_dots_slashes() {
            assert_eq!(trim_leading_dots_slashes("./dependencies"), "dependencies");
            assert_eq!(trim_leading_dots_slashes(".\\dependencies"), "dependencies");
            assert_eq!(trim_leading_dots_slashes("dependencies"), "dependencies");
        }

        // ─── split_semicolon_list ─────────────────────────────────────────

        #[test]
        fn test_split_semicolon_list_basic() {
            let items = split_semicolon_list(r".\src;.\dep\src;$(DCC_UnitSearchPath)");
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], r".\src");
        }

        #[test]
        fn test_split_semicolon_list_ignores_empty_segments() {
            let items = split_semicolon_list(r".\src;;.\dep");
            assert_eq!(items.len(), 2);
        }

        // ─── merge_semicolon_paths ────────────────────────────────────────

        #[test]
        fn test_merge_semicolon_paths_no_duplicates() {
            let mut dst = vec![r".\src".to_string()];
            let more = vec![r".\src".to_string(), r".\dep\src".to_string()];
            merge_semicolon_paths(&mut dst, &more);
            assert_eq!(dst.len(), 2);
            assert!(dst.contains(&r".\dep\src".to_string()));
        }

        // ─── is_under_top_level_src ───────────────────────────────────────

        #[test]
        fn test_is_under_top_level_src_true() {
            let parts = vec!["package".to_string(), "src".to_string()];
            assert!(is_under_top_level_src(&parts));
            let parts = vec!["package".to_string(), "Source".to_string()];
            assert!(is_under_top_level_src(&parts));
        }

        #[test]
        fn test_is_under_top_level_src_false() {
            let parts = vec!["package".to_string(), "bin".to_string()];
            assert!(!is_under_top_level_src(&parts));
            assert!(!is_under_top_level_src(&["only_one".to_string()]));
        }

        // ─── collect_dependency_paths ─────────────────────────────────────

        /// Finds the direct `src` directory inside a package.
        #[test]
        fn test_collect_dependency_paths_finds_src_dir() {
            let dir = TempDir::new().unwrap();
            fs::create_dir_all(dir.path().join("Horse/src")).unwrap();

            let paths = collect_dependency_paths(dir.path().to_str().unwrap()).unwrap();
            let joined = paths.join(";");
            assert!(joined.contains("Horse"), "should find Horse/src");
            assert!(joined.contains("src"), "should reference src segment");
        }

        /// Accepts `Source` (capitalised) as an alternative to `src`.
        #[test]
        fn test_collect_dependency_paths_finds_source_dir() {
            let dir = TempDir::new().unwrap();
            fs::create_dir_all(dir.path().join("Nidus/Source")).unwrap();

            let paths = collect_dependency_paths(dir.path().to_str().unwrap()).unwrap();
            let joined = paths.join(";");
            assert!(joined.contains("Nidus"), "should find Nidus/Source");
        }

        /// Includes the `src` folder itself (depth 2).
        #[test]
        fn test_collect_dependency_paths_includes_src_itself() {
            let dir = TempDir::new().unwrap();
            fs::create_dir_all(dir.path().join("Horse/src")).unwrap();

            let paths = collect_dependency_paths(dir.path().to_str().unwrap()).unwrap();
            let joined = paths.join(";");
            assert!(
                joined.contains(r"Horse\src") || joined.contains("Horse/src"),
                "Horse\\src must be in paths; got: {:?}", paths
            );
        }

        /// Includes first-level subfolders inside `src` (depth 3).
        #[test]
        fn test_collect_dependency_paths_includes_direct_src_subdirs() {
            let dir = TempDir::new().unwrap();
            fs::create_dir_all(dir.path().join("Horse/src/Pipes")).unwrap();
            fs::create_dir_all(dir.path().join("Horse/src/Core")).unwrap();

            let paths = collect_dependency_paths(dir.path().to_str().unwrap()).unwrap();
            let joined = paths.join(";");

            // Must contain src itself (implicitly created by create_dir_all)
            assert!(
                joined.contains(r"Horse\src") || joined.contains("Horse/src"),
                "Horse\\src must be present; got: {:?}", paths
            );
            // Must contain each direct subfolder
            assert!(
                joined.contains("Pipes"),
                "Horse\\src\\Pipes must be present; got: {:?}", paths
            );
            assert!(
                joined.contains("Core"),
                "Horse\\src\\Core must be present; got: {:?}", paths
            );
            // Contagem: Horse/src + Horse/src/Pipes + Horse/src/Core = 3
            assert_eq!(paths.len(), 3, "expected exactly 3 paths; got: {:?}", paths);
        }

        /// Includes subfolders at depth 3+ (level 4 and beyond).
        #[test]
        fn test_collect_dependency_paths_includes_deeply_nested_subdirs() {
            let dir = TempDir::new().unwrap();
            // Structure: Horse/src/Core/Utils/Helpers  (4 levels below root)
            fs::create_dir_all(dir.path().join("Horse/src/Core/Utils/Helpers")).unwrap();

            let paths = collect_dependency_paths(dir.path().to_str().unwrap()).unwrap();
            let joined = paths.join(";");

            assert!(joined.contains("Core"),    "Core must be present");
            assert!(joined.contains("Utils"),   "Utils must be present");
            assert!(joined.contains("Helpers"), "Helpers must be present");
            // src + Core + Utils + Helpers = 4
            assert_eq!(paths.len(), 4, "all depth levels must be collected; got: {:?}", paths);
        }

        /// Ignores folders that are NOT `src`/`Source` (e.g., `bin`, `test`).
        #[test]
        fn test_collect_dependency_paths_ignores_non_src_dirs() {
            let dir = TempDir::new().unwrap();
            fs::create_dir_all(dir.path().join("Horse/bin")).unwrap();
            fs::create_dir_all(dir.path().join("Horse/test")).unwrap();

            let paths = collect_dependency_paths(dir.path().to_str().unwrap()).unwrap();
            assert!(paths.is_empty(), "non-src dirs must be ignored; got: {:?}", paths);
        }

        /// Empty `dependencies` folder returns an empty list without error.
        #[test]
        fn test_collect_dependency_paths_empty_dir_returns_empty() {
            let dir = TempDir::new().unwrap();
            let paths = collect_dependency_paths(dir.path().to_str().unwrap()).unwrap();
            assert!(paths.is_empty());
        }

        /// Múltiplos pacotes são todos coletados.
        #[test]
        fn test_collect_dependency_paths_multiple_packages() {
            let dir = TempDir::new().unwrap();
            fs::create_dir_all(dir.path().join("Horse/src")).unwrap();
            fs::create_dir_all(dir.path().join("Nidus/Source")).unwrap();
            fs::create_dir_all(dir.path().join("InjectContainer/src")).unwrap();

            let paths = collect_dependency_paths(dir.path().to_str().unwrap()).unwrap();
            let joined = paths.join(";");
            assert!(joined.contains("Horse"),           "Horse/src must be found");
            assert!(joined.contains("Nidus"),           "Nidus/Source must be found");
            assert!(joined.contains("InjectContainer"), "InjectContainer/src must be found");
        }

        // ─── fix_msbuild_header ───────────────────────────────────────────

        /// Removes the `<?xml ...?>` header when injected by the writer.
        #[test]
        fn test_fix_msbuild_header_strips_xml_declaration() {
            let dir = TempDir::new().unwrap();
            let path = dir.path().join("MyApp.dproj");
            fs::write(
                &path,
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<Project></Project>",
            ).unwrap();

            fix_msbuild_header(path.to_str().unwrap()).unwrap();

            let content = fs::read_to_string(&path).unwrap();
            assert!(!content.starts_with("<?xml"), "XML header must be stripped");
            assert!(content.contains("<Project>"), "project content must be preserved");
        }

        /// File without a header is not modified.
        #[test]
        fn test_fix_msbuild_header_no_op_when_no_header() {
            let dir = TempDir::new().unwrap();
            let path = dir.path().join("MyApp.dproj");
            let original = "<Project></Project>";
            fs::write(&path, original).unwrap();

            fix_msbuild_header(path.to_str().unwrap()).unwrap();

            let content = fs::read_to_string(&path).unwrap();
            assert_eq!(content, original, "file without header must be unchanged");
        }

        // ─── add_search_paths_to_dproj ────────────────────────────────────

        #[test]
        fn test_add_search_paths_creates_entry() {
            let dir = TempDir::new().unwrap();
            let dproj_path = dir.path().join("MyApp.dproj");
            fs::write(&dproj_path, dproj_without_search_path()).unwrap();

            let result = add_search_paths_to_dproj(
                dproj_path.to_str().unwrap(),
                &[r".\dependencies\Horse\src"],
            );
            assert!(result.is_ok(), "failed: {:?}", result.err());

            let content = fs::read_to_string(&dproj_path).unwrap();
            assert!(content.contains("Horse"));
        }

        #[test]
        fn test_add_search_paths_appends_to_existing() {
            let dir = TempDir::new().unwrap();
            let dproj_path = dir.path().join("MyApp.dproj");
            fs::write(
                &dproj_path,
                minimal_dproj(r".\existing\src;$(DCC_UnitSearchPath)"),
            ).unwrap();

            add_search_paths_to_dproj(
                dproj_path.to_str().unwrap(),
                &[r".\dependencies\NewLib\src"],
            ).unwrap();

            let content = fs::read_to_string(&dproj_path).unwrap();
            assert!(content.contains("existing"));
            assert!(content.contains("NewLib"));
        }

        #[test]
        fn test_add_search_paths_no_duplicates() {
            let dir = TempDir::new().unwrap();
            let dproj_path = dir.path().join("MyApp.dproj");
            let existing = r".\dependencies\Horse\src";
            fs::write(&dproj_path, minimal_dproj(existing)).unwrap();

            add_search_paths_to_dproj(
                dproj_path.to_str().unwrap(),
                &[existing],
            ).unwrap();

            let content = fs::read_to_string(&dproj_path).unwrap();
            assert_eq!(content.matches(r"Horse\src").count(), 1);
        }

        #[test]
        fn test_add_search_paths_macro_placed_last() {
            let dir = TempDir::new().unwrap();
            let dproj_path = dir.path().join("MyApp.dproj");
            fs::write(&dproj_path, dproj_without_search_path()).unwrap();

            add_search_paths_to_dproj(
                dproj_path.to_str().unwrap(),
                &[r".\dependencies\Horse\src"],
            ).unwrap();

            let content = fs::read_to_string(&dproj_path).unwrap();
            let macro_pos = content.find("$(DCC_UnitSearchPath)").unwrap();
            let horse_pos = content.find("Horse").unwrap();
            assert!(horse_pos < macro_pos, "macro should come after paths");
        }

        /// Multiple paths added at once are all present in the .dproj.
        #[test]
        fn test_add_search_paths_multiple_paths_at_once() {
            let dir = TempDir::new().unwrap();
            let dproj_path = dir.path().join("MyApp.dproj");
            fs::write(&dproj_path, dproj_without_search_path()).unwrap();

            add_search_paths_to_dproj(
                dproj_path.to_str().unwrap(),
                &[
                    r".\dependencies\Horse\src",
                    r".\dependencies\Nidus\Source",
                    r".\dependencies\InjectContainer\src",
                ],
            ).unwrap();

            let content = fs::read_to_string(&dproj_path).unwrap();
            assert!(content.contains("Horse"),           "Horse path must be present");
            assert!(content.contains("Nidus"),           "Nidus path must be present");
            assert!(content.contains("InjectContainer"), "InjectContainer path must be present");
        }

        /// Repeated calls with the same paths do not duplicate entries.
        #[test]
        fn test_add_search_paths_idempotent() {
            let dir = TempDir::new().unwrap();
            let dproj_path = dir.path().join("MyApp.dproj");
            fs::write(&dproj_path, dproj_without_search_path()).unwrap();
            let path = r".\dependencies\Horse\src";

            add_search_paths_to_dproj(dproj_path.to_str().unwrap(), &[path]).unwrap();
            add_search_paths_to_dproj(dproj_path.to_str().unwrap(), &[path]).unwrap();

            let content = fs::read_to_string(&dproj_path).unwrap();
            assert_eq!(content.matches(r"Horse\src").count(), 1, "idempotent: no duplicates on re-run");
        }

        /// `.\` is preserved as the first entry when the node is created from scratch.
        #[test]
        fn test_add_search_paths_dot_backslash_first_when_created() {
            let dir = TempDir::new().unwrap();
            let dproj_path = dir.path().join("MyApp.dproj");
            fs::write(&dproj_path, dproj_without_search_path()).unwrap();

            add_search_paths_to_dproj(
                dproj_path.to_str().unwrap(),
                &[r".\dependencies\Horse\src"],
            ).unwrap();

            let content = fs::read_to_string(&dproj_path).unwrap();
            // The node value must start with .\
            let node_content = content
                .split("<DCC_UnitSearchPath>")
                .nth(1)
                .and_then(|s| s.split("</DCC_UnitSearchPath>").next())
                .unwrap_or("");
            assert!(
                node_content.trim_start().starts_with(r".\"),
                r"first entry must be .\ but got: {}",
                node_content
            );
        }
    }

}
