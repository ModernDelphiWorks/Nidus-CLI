pub mod module_unit {
    use crate::core::core_utils::utils;
    use regex::Regex;
    use std::path::Path;

    /// 🇧🇷 Atualiza o `AppModule.pas` inserindo units no `uses`, substituindo `<mod>`,
    /// e upsert em `RouteHandlers` e `Routes`.
    /// 🇺🇸 Updates `AppModule.pas` by inserting units in `uses`, replacing `<mod>`,
    /// and upserting into `RouteHandlers` and `Routes`.
    pub fn update_app_module(module_name: &str, generated: &[&str]) -> std::io::Result<()> {
        let app_module_path = Path::new("src").join("AppModule.pas");
        let mut content = std::fs::read_to_string(&app_module_path)?;

        let mod_camel = utils::camel_case(module_name);
        let mod_lower = module_name.to_lowercase();

        _add_units_to_uses(&mut content, &mod_camel, generated);
        _replace_module_placeholder(&mut content, &mod_camel);
        _upsert_route_handlers(&mut content, &mod_camel, generated);
        _upsert_routes(&mut content, &mod_camel, &mod_lower, generated);

        std::fs::write(app_module_path, content)?;
        Ok(())
    }

    // ========== 1) USES ==========

    /// 🇧🇷 Insere `T<Mod>Module` e/ou `T<Mod>Handler` no bloco `uses` (sem duplicar).
    /// 🇺🇸 Inserts `T<Mod>Module` and/or `T<Mod>Handler` into the `uses` block (no duplicates).
    fn _add_units_to_uses(content: &mut String, mod_camel: &str, generated: &[&str]) {
        for comp in ["module", "handler"] {
            if generated.contains(&comp) {
                let unit_name = format!("{}{}", mod_camel, utils::camel_case(comp));
                if content.contains(&unit_name) {
                    continue;
                }
                if let Some(insert_pos) = _find_uses_insert_pos(content) {
                    content.insert_str(insert_pos, &format!(",\n  {}", unit_name));
                } else {
                    eprintln!(
                        "⚠️  Could not find 'uses' block in AppModule.pas — add '{}' manually.",
                        unit_name
                    );
                }
            }
        }
    }

    /// 🇧🇷 Encontra a posição logo antes do `;` do primeiro `uses` para inserir novas units.
    /// 🇺🇸 Finds the position right before `;` of the first `uses` to insert new units.
    fn _find_uses_insert_pos(content: &str) -> Option<usize> {
        let idx_uses = content.find("uses")?;
        let rest = &content[idx_uses..];
        let end_idx = rest.find(';')?;
        Some(idx_uses + end_idx)
    }

    // ========== 2) PLACEHOLDER ==========

    /// 🇧🇷 Substitui `<mod>` pelo nome do módulo em CamelCase.
    /// 🇺🇸 Replaces `<mod>` with the module name in CamelCase.
    fn _replace_module_placeholder(content: &mut String, mod_camel: &str) {
        *content = content.replace("<mod>", mod_camel);
    }

    // ========== 3) ROUTE HANDLERS ==========

    /// 🇧🇷 Faz upsert de `T<Mod>Handler` no método `TAppModule.RouteHandlers`.
    /// 🇺🇸 Upserts `T<Mod>Handler` into `TAppModule.RouteHandlers`.
    fn _upsert_route_handlers(content: &mut String, mod_camel: &str, generated: &[&str]) {
        if generated.contains(&"handler") {
            upsert_result_list_aligned(
                content,
                "RouteHandlers",
                &[format!("T{}RouteHandler", mod_camel)],
            );
        }
    }

    // ========== 4) ROUTES ==========

    /// 🇧🇷 Faz upsert de `RouteModule('/api/v1/<mod>', T<Mod>Module)` no método `TAppModule.Routes`.
    /// 🇺🇸 Upserts `RouteModule('/api/v1/<mod>', T<Mod>Module)` into `TAppModule.Routes`.
    fn _upsert_routes(content: &mut String, mod_camel: &str, mod_lower: &str, generated: &[&str]) {
        if generated.contains(&"module") {
            upsert_result_list_aligned(
                content,
                "Routes",
                &[format!(
                    "RouteModule('/api/v1/{}', T{}Module)",
                    mod_lower, mod_camel
                )],
            );
        }
    }

    // ========== CORE: Upsert + Alinhamento ==========

    /// 🇧🇷 Junta itens (sem duplicar) e REFORMATA o `Result := [...]` do método
    /// `function TAppModule.<func_name>` no estilo alinhado.
    /// 🇺🇸 Merges items (no duplicates) and REFORMATS the `Result := [...]` of
    /// `function TAppModule.<func_name>` using aligned style.
    fn upsert_result_list_aligned(content: &mut String, func_name: &str, add_items: &[String]) {
        if add_items.is_empty() {
            return;
        }

        // Captura o bloco da função inteira
        let re_func = Regex::new(&format!(
            r"(?s)(function\s+TAppModule\.{}\b.*?begin)(.*?)(end;)",
            regex::escape(func_name)
        ))
        .unwrap();

        let caps = match re_func.captures(content) {
            Some(c) => c,
            None => return, // função não encontrada — não faz nada
        };

        let full = caps.get(0).unwrap();
        let head = caps.get(1).unwrap().as_str();
        let body = caps.get(2).unwrap().as_str();
        let tail = caps.get(3).unwrap().as_str();

        // Capture the content of Result := [ ... ];
        let re_result = Regex::new(r"(?s)Result\s*:=\s*\[(?P<items>.*?)\]\s*;").unwrap();

        let mut items = parse_result_items(body, &re_result);

        // Add without duplicating
        for it in add_items {
            if !items.iter().any(|x| x == it) {
                items.push(it.clone());
            }
        }

        // Reconstroi bloco `Result := [ ... ];` alinhado e limpa o antigo
        let rebuilt_result = rebuild_aligned_result(&items);
        let body_clean = re_result.replace_all(body, "").to_string();

        let new_body = format!("\n{}{}", rebuilt_result, body_clean.trim_start());
        let new_block = format!("{}{}{}", head, new_body, tail);
        content.replace_range(full.start()..full.end(), &new_block);
    }

    /// 🇧🇷 Lê os itens atuais de `Result := [ ... ];` respeitando parênteses.
    /// 🇺🇸 Parses current items from `Result := [ ... ];` honoring parentheses.
    fn parse_result_items(body: &str, re_result: &Regex) -> Vec<String> {
        let mut items = Vec::new();
        if let Some(rc) = re_result.captures(body) {
            let inside = rc.name("items").unwrap().as_str();
            let mut current = String::new();
            let mut depth = 0;
            for ch in inside.chars() {
                match ch {
                    '(' => {
                        depth += 1;
                        current.push(ch);
                    }
                    ')' => {
                        depth -= 1;
                        current.push(ch);
                    }
                    ',' if depth == 0 => {
                        let t = current.trim();
                        if !t.is_empty() && !items.iter().any(|x| x == t) {
                            items.push(t.to_string());
                        }
                        current.clear();
                    }
                    _ => current.push(ch),
                }
            }
            let t = current.trim();
            if !t.is_empty() && !items.iter().any(|x| x == t) {
                items.push(t.to_string());
            }
        }
        items
    }

    #[cfg(test)]
    pub(super) fn rebuild_aligned_result_pub(items: &[String]) -> String {
        rebuild_aligned_result(items)
    }

    #[cfg(test)]
    pub(super) fn find_uses_insert_pos_pub(content: &str) -> Option<usize> {
        _find_uses_insert_pos(content)
    }

    #[cfg(test)]
    pub(super) fn replace_module_placeholder_pub(content: &mut String, mod_camel: &str) {
        _replace_module_placeholder(content, mod_camel)
    }

    #[cfg(test)]
    pub(super) fn upsert_result_list_aligned_pub(content: &mut String, func_name: &str, items: &[String]) {
        upsert_result_list_aligned(content, func_name, items)
    }

    /// 🇧🇷 Recria o `Result := [ ... ];` com indentação e quebras alinhadas.
    /// 🇺🇸 Rebuilds `Result := [ ... ];` using aligned indentation and breaks.
    fn rebuild_aligned_result(items: &[String]) -> String {
        let result_line_indent = "  ";
        let result_prefix = "Result := [";
        let align_spaces = " ".repeat(result_line_indent.len() + result_prefix.len());

        if items.is_empty() {
            return format!("{}{}];\n", result_line_indent, result_prefix);
        }

        let mut format_result = String::new();
        format_result.push_str(result_line_indent);
        format_result.push_str(result_prefix);
        format_result.push_str(&items[0]);

        for it in &items[1..] {
            format_result.push_str(",\n");
            format_result.push_str(&align_spaces);
            format_result.push_str(it);
        }
        format_result.push_str("];\n");
        format_result
    }
}

#[cfg(test)]
mod tests {
    use super::module_unit::*;

    #[test]
    fn test_find_uses_insert_pos_basic() {
        let content = "uses\n  SomeUnit;\n";
        let pos = find_uses_insert_pos_pub(content).unwrap();
        // Deve apontar para o ';' final do uses
        assert_eq!(&content[pos..pos + 1], ";");
    }

    #[test]
    fn test_find_uses_insert_pos_no_uses() {
        assert!(find_uses_insert_pos_pub("no uses block here").is_none());
    }

    #[test]
    fn test_replace_module_placeholder() {
        let mut content = "T<mod>Module registers T<mod>Service".to_string();
        replace_module_placeholder_pub(&mut content, "User");
        assert_eq!(content, "TUserModule registers TUserService");
    }

    #[test]
    fn test_replace_module_placeholder_no_match() {
        let mut content = "nothing to replace".to_string();
        replace_module_placeholder_pub(&mut content, "User");
        assert_eq!(content, "nothing to replace");
    }

    #[test]
    fn test_rebuild_aligned_result_empty() {
        let result = rebuild_aligned_result_pub(&[]);
        assert!(result.contains("Result := ["));
        assert!(result.contains("];"));
    }

    #[test]
    fn test_rebuild_aligned_result_single_item() {
        let items = vec!["TUserHandler".to_string()];
        let result = rebuild_aligned_result_pub(&items);
        assert!(result.contains("TUserHandler"));
        assert!(result.contains("Result := ["));
    }

    #[test]
    fn test_rebuild_aligned_result_multiple_items() {
        let items = vec!["TUserHandler".to_string(), "TOrderHandler".to_string()];
        let result = rebuild_aligned_result_pub(&items);
        assert!(result.contains("TUserHandler"));
        assert!(result.contains("TOrderHandler"));
        assert!(result.contains(",\n"));
    }

    #[test]
    fn test_upsert_adds_item_to_function() {
        let mut content = "\
function TAppModule.RouteHandlers: TArray<TObject>;
begin
  Result := [];
end;
"
        .to_string();
        upsert_result_list_aligned_pub(
            &mut content,
            "RouteHandlers",
            &["TUserHandler".to_string()],
        );
        assert!(content.contains("TUserHandler"));
    }

    #[test]
    fn test_upsert_no_duplicates() {
        let mut content = "\
function TAppModule.Routes: TArray<TObject>;
begin
  Result := [RouteModule('/api/v1/user', TUserModule)];
end;
"
        .to_string();
        upsert_result_list_aligned_pub(
            &mut content,
            "Routes",
            &["RouteModule('/api/v1/user', TUserModule)".to_string()],
        );
        // Deve conter apenas uma ocorrência
        assert_eq!(
            content.matches("RouteModule('/api/v1/user', TUserModule)").count(),
            1
        );
    }
}
