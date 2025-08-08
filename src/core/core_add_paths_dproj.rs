pub mod dproj {
    use etree::{ETree, ETreeNode};
    use std::{collections::HashSet, error::Error, fs, path::Path};
    use walkdir::{IntoIter, WalkDir};

    // =============================
    // Public Orchestration
    // =============================

    /// 🇧🇷 Encontra .dproj(s) no diretório atual e caminhos de dependências (src/Source).
    /// 🇺🇸 Finds .dproj files in CWD and dependency paths (src/Source).
    pub fn find_dproj_files_and_collect_paths() -> Result<(Vec<String>, Vec<String>), Box<dyn Error>>
    {
        let dprojs = find_dproj_in_cwd()?;
        if dprojs.is_empty() {
            return Err("No .dproj found in current directory".into());
        }

        let dep_paths = collect_dependency_paths("./dependencies")?;
        Ok((dprojs, dep_paths))
    }

    /// 🇧🇷 Adiciona (mescla) caminhos ao `DCC_UnitSearchPath` do .dproj.
    /// 🇺🇸 Adds (merges) paths into .dproj's `DCC_UnitSearchPath`.
    pub fn add_search_paths_to_dproj(
        dproj_path: &str,
        additional_paths: &[&str],
    ) -> Result<(), Box<dyn Error>> {
        const XML_TAG_PROPERTY_GROUP: &str = "PropertyGroup";
        const XML_ATTR_CONDITION: &str = "Condition";
        const XML_ATTR_CONDITION_VALUE: &str = r##"'$(Base)'!=''"##; // bloco “Base”
        const XML_TAG_LIBRARY_PATH: &str = "DCC_UnitSearchPath";
        const MACRO: &str = "$(DCC_UnitSearchPath)";

        // Carrega XML
        let mut etree = ETree::parse_file(dproj_path);
        let root = etree.root();

        // Localiza PropertyGroup com a Condition esperada
        let property_group_pos = find_property_group(
            &etree,
            root,
            XML_TAG_PROPERTY_GROUP,
            XML_ATTR_CONDITION,
            XML_ATTR_CONDITION_VALUE,
        )
        .ok_or("PropertyGroup with expected Condition not found")?;

        // Garante existência do nó DCC_UnitSearchPath no *final* do PropertyGroup
        let (dcc_pos, created_now) =
            get_or_create_child_node_at_end(&mut etree, property_group_pos, XML_TAG_LIBRARY_PATH)?;

        // Texto atual (se houver)
        let current_text = etree
            .node(dcc_pos)
            .and_then(|n| n.get_text())
            .unwrap_or_default();

        // Quebrar atual em itens
        let mut items = split_semicolon_list(&current_text);

        // Detecta .\ existente antes de mexer (vamos preservar se existir ou se criarmos do zero)
        let had_root_dot_any = items.iter().any(|s| s.trim() == r".\");

        // Mesclar novos caminhos normalizados
        let add_norm: Vec<String> = additional_paths
            .iter()
            .map(|p| normalize_windows_path(p))
            .collect();
        merge_semicolon_paths(&mut items, &add_norm);

        // Se criou agora ou já havia .\, manter .\ como primeiro
        if created_now || had_root_dot_any {
            // Remove ocorrências antigas de .\
            items.retain(|s| s.trim() != r".\");
            // Insere na frente
            items.insert(0, r".\".to_string());
        }

        // Garantir macro no final, uma única vez
        let has_macro = items.iter().any(|s| s.trim().eq_ignore_ascii_case(MACRO));
        if !has_macro {
            items.push(MACRO.to_string());
        } else if !items
            .last()
            .map(|s| s.trim().eq_ignore_ascii_case(MACRO))
            .unwrap_or(false)
        {
            // Move macro pro final
            let mut without_macro: Vec<String> = items
                .into_iter()
                .filter(|s| !s.trim().eq_ignore_ascii_case(MACRO))
                .collect();
            without_macro.push(MACRO.to_string());
            items = without_macro;
        }

        // Escrever de volta
        etree.node_mut(dcc_pos).unwrap().set_text(&items.join(";"));
        etree.write_file(dproj_path)?;

        // ✅ Pós-processamento: remover header XML acidental (<?xml ...?>) para manter a 1ª linha do Delphi
        fix_msbuild_header(dproj_path)?;

        Ok(())
    }

    // =============================
    // Layer 1: Project discovery
    // =============================

    /// 🇧🇷 Procura arquivos .dproj no diretório atual.
    /// 🇺🇸 Finds .dproj files in the current directory.
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

    /// 🇧🇷 Varre ./dependencies e retorna SOMENTE paths sob {pacote}\(src|Source)\** (case-insensitive).
    /// 🇺🇸 Scans ./dependencies and returns ONLY paths under {package}\(src|Source)\** (case-insensitive).
    fn collect_dependency_paths(root: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let mut set: HashSet<String> = HashSet::new();

        let entries: IntoIter = WalkDir::new(root).into_iter();
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            if let Ok(relative) = path.strip_prefix(root) {
                // Segmentos: [pacote, src|Source, ...]
                let parts: Vec<String> = relative
                    .iter()
                    .filter_map(|p| p.to_str())
                    .map(|s| s.to_string())
                    .collect();

                // ✅ Aceitar apenas quando:
                // - há pelo menos 2 segmentos
                // - o segundo segmento é "src" ou "Source" (case-insensitive)
                if is_under_top_level_src(&parts) {
                    // Monta caminho no padrão .\dependencies\{pacote}\(src|Source)\...
                    let win_rel = to_win_path(relative); // "nest4d\Source\Pipes\..."
                    let trimmed_root = trim_leading_dots_slashes(root); // "dependencies"

                    // ✅ sempre insere UMA barra entre root e relativo
                    let full = format!(r".\{}\{}", trimmed_root, win_rel.trim_start_matches('\\'));

                    // normaliza (sem remover barras simples)
                    set.insert(normalize_windows_path(&full));
                }
            }
        }

        Ok(set.into_iter().collect())
    }

    /// 🇧🇷 Verifica se `parts` está sob {pacote}\(src|Source)\...
    /// 🇺🇸 Checks whether `parts` lies under {package}\(src|Source)\...
    fn is_under_top_level_src(parts: &[String]) -> bool {
        if parts.len() < 2 {
            return false;
        }
        let second = parts[1].to_ascii_lowercase();
        second == "src" || second == "source"
    }

    /// 🇧🇷 Converte `Path` relativo para notação com `\` (Windows).
    /// 🇺🇸 Converts relative `Path` into `\` notation (Windows).
    fn to_win_path(p: &Path) -> String {
        p.to_string_lossy().replace('/', r"\").to_string()
    }

    /// 🇧🇷 Remove prefixos `./`, `.\\` e barras.
    /// 🇺🇸 Removes leading `./`, `.\\` and slashes.
    fn trim_leading_dots_slashes(s: &str) -> String {
        s.trim_start_matches("./")
            .trim_start_matches(".\\")
            .trim_start_matches('\\')
            .trim_start_matches('/')
            .to_string()
    }

    /// 🇧🇷 Normaliza para separador `\`, remove duplicidades e `;` finais.
    /// 🇺🇸 Normalizes to `\`, removes duplicates and trailing `;`.
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

    /// 🇧🇷 Encontra `PropertyGroup` com `Condition == expected_value`.
    /// 🇺🇸 Finds `PropertyGroup` with `Condition == expected_value`.
    fn find_property_group(
        etree: &ETree,
        start: usize,
        tag_name: &str,
        attr_name: &str,
        expected_value: &str,
    ) -> Option<usize> {
        let mut stack = vec![start];
        while let Some(pos) = stack.pop() {
            if let Some(node) = etree.node(pos) {
                if node.get_name() == tag_name {
                    if let Some(val) = node.get_attr(attr_name) {
                        if val == expected_value {
                            return Some(pos);
                        }
                    }
                }
                stack.extend(etree.children(pos));
            }
        }
        None
    }

    /// 🇧🇷 Obtém (ou cria) um nó filho pelo nome, inserindo no *final*, e indica se foi criado agora.
    /// 🇺🇸 Gets (or creates) a child node by name, appending at the *end*, returning position and creation flag.
    fn get_or_create_child_node_at_end(
        etree: &mut ETree,
        parent_pos: usize,
        child_name: &str,
    ) -> Result<(usize, bool), Box<dyn Error>> {
        if let Some(first) = etree
            .children_by_name(parent_pos, child_name)
            .first()
            .copied()
        {
            return Ok((first, false));
        }
        let new_node = ETreeNode::new(child_name);
        let pos = etree
            .append_child_node(parent_pos, new_node)
            .ok_or("Failed to append child node")?;
        Ok((pos, true))
    }

    /// 🇧🇷 Separa lista `a;b;c` preservando ordem e ignorando vazios.
    /// 🇺🇸 Splits `a;b;c` list preserving order and ignoring empties.
    fn split_semicolon_list(s: &str) -> Vec<String> {
        s.split(';')
            .map(str::trim)
            .filter(|x| !x.is_empty())
            .map(|x| x.to_string())
            .collect()
    }

    /// 🇧🇷 Mescla `more` em `dst` sem duplicar, preservando ordem original de `dst`.
    /// 🇺🇸 Merges `more` into `dst` without duplicates, preserving original `dst` order.
    fn merge_semicolon_paths(dst: &mut Vec<String>, more: &[String]) {
        let set: HashSet<String> = dst.iter().cloned().collect();
        for m in more {
            if !set.contains(m) && !dst.iter().any(|x| x.eq_ignore_ascii_case(m)) {
                dst.push(m.clone());
            }
        }
    }

    /// 🇧🇷 Remove uma linha XML header `<?xml ...?>` caso tenha sido inserida pelo writer.
    /// 🇺🇸 Strips XML header `<?xml ...?>` if writer inserted it.
    fn fix_msbuild_header(path: &str) -> Result<(), Box<dyn Error>> {
        let mut text = fs::read_to_string(path)?;
        if text.starts_with("<?xml") {
            // remove header e espaços/brancas subsequentes
            if let Some(end) = text.find("?>") {
                let after = &text[end + 2..];
                let new_text = after
                    .trim_start_matches(|c| c == '\u{feff}' || c == '\r' || c == '\n' || c == ' ');
                text = new_text.to_string();
                fs::write(path, &text)?;
            }
        }
        // Não forçamos a primeira linha — assumimos que já é <Project ...> do arquivo original
        Ok(())
    }

    // =============================
    // (Opcional) Uma função “faça tudo”
    // =============================

    /// 🇧🇷 Fluxo completo: encontra .dproj, coleta paths e atualiza todos.
    /// 🇺🇸 Full flow: find .dproj, collect paths and update all of them.
    pub fn update_all_dprojs_in_cwd() -> Result<(), Box<dyn Error>> {
        let (dprojs, deps) = find_dproj_files_and_collect_paths()?;
        if dprojs.is_empty() {
            return Err("No .dproj found".into());
        }
        let deps_ref: Vec<&str> = deps.iter().map(|s| s.as_str()).collect();
        for d in &dprojs {
            add_search_paths_to_dproj(d, &deps_ref)?;
        }
        Ok(())
    }
}
