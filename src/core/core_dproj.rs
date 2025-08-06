pub mod dproj {
    use std::{
        collections::HashSet,
        fs::{self, ReadDir},
        path::{Path, PathBuf},
    };

    use etree::{ETree, ETreeNode};
    use walkdir::{IntoIter, WalkDir};

    pub fn add_search_paths_to_dproj(
        dproj_path: &str,
        additional_paths: &[&str],
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Define constantes para os nomes das tags XML e atributos
        const XML_TAG_NAME_PROPERTY: &str = "PropertyGroup";
        const XML_TAG_NAME_PROPERTY_ATTRIBUTE: &str = "Condition";
        const XML_TAG_NAME_PROPERTY_ATTRIBUTE_VALUE: &str = r##"'$(Base)'!=''"##;
        const XML_TAG_NAME_LIBRARY_PATH: &str = "DCC_UnitSearchPath";
        // Carregar o arquivo .dproj como uma árvore de elementos XML
        let mut etree: ETree = ETree::parse_file(dproj_path);
        // Obter a posição do nó raiz
        let root_pos: usize = etree.root();
        // Variável para armazenar a posição do PropertyGroup encontrado
        let mut property_group_pos: Option<usize> = None;
        // Pilha para armazenar nós a serem processados
        let mut stack: Vec<usize> = vec![root_pos];
        // Percorrer a árvore de nós XML
        while let Some(pos) = stack.pop() {
            // Obter o nó atual
            let node: &ETreeNode = etree.node(pos).unwrap();
            // Verificar se o nó é um PropertyGroup
            if node.get_name() == XML_TAG_NAME_PROPERTY {
                // Verificar se o nó possui o atributo Condition com o valor desejado
                if let Some(condition_attr_value) = node.get_attr(XML_TAG_NAME_PROPERTY_ATTRIBUTE) {
                    if condition_attr_value == XML_TAG_NAME_PROPERTY_ATTRIBUTE_VALUE {
                        // Encontramos o PropertyGroup desejado, armazenar sua posição
                        property_group_pos = Some(pos);
                        break;
                    }
                }
            }
            // Adicionar filhos do nó atual à pilha para processamento
            stack.extend(etree.children(pos));
        }
        // Verificar se o PropertyGroup foi encontrado, caso contrário, encerra o processo
        let property_group_pos: usize = match property_group_pos {
            Some(pos) => pos,
            None => {
                println!("🚨 PropertyGroup not found!");
                std::process::exit(0);
            }
        };
        // Encontrar o elemento DCC_UnitSearchPath dentro do PropertyGroup encontrado
        let dcc_unit_search_path_pos: Option<usize> = etree
            .children_by_name(property_group_pos, XML_TAG_NAME_LIBRARY_PATH)
            .first()
            .copied();
        // Obter uma referência mutável para o elemento DCC_UnitSearchPath
        let dcc_unit_search_path_node: &mut ETreeNode = if let Some(pos) = dcc_unit_search_path_pos
        {
            // Se o elemento existir, obter sua posição
            etree.node_mut(pos).unwrap()
        } else {
            // Caso contrário, criar um novo elemento e obter sua posição
            let new_node: ETreeNode = ETreeNode::new(XML_TAG_NAME_LIBRARY_PATH);
            let pos: usize = etree
                .append_child_node(property_group_pos, new_node)
                .unwrap();
            etree.node_mut(pos).unwrap()
        };
        // Obter os caminhos atuais do elemento DCC_UnitSearchPath
        let current_paths: String = dcc_unit_search_path_node.get_text().unwrap_or_default();
        // Separar os caminhos em um vetor
        let mut paths: Vec<&str> = current_paths.split(';').collect();
        // Adicionar os novos caminhos ao vetor apenas se ainda não existirem
        for path in additional_paths {
            if !paths.contains(path) {
                paths.push(path);
            }
        }
        // Atualizar o texto do elemento DCC_UnitSearchPath com os caminhos combinados
        dcc_unit_search_path_node.set_text(&paths.join(";"));
        // Salvar a árvore XML modificada de volta para o arquivo .dproj
        etree.write_file(dproj_path)?;

        Ok(())
    }

    pub fn find_dproj_files_and_collect_paths() -> (Vec<String>, Vec<String>) {
        let mut dproj_files: Vec<String> = Vec::new();
        let mut dependency_paths: HashSet<String> = HashSet::new();

        // Procurar arquivos .dproj no diretório atual
        let entries: ReadDir = fs::read_dir(".").expect("Failed to read directory");
        for entry in entries.flatten() {
            let path: PathBuf = entry.path();
            if let Some(extension) = path.extension() {
                if extension == "dproj" {
                    dproj_files.push(path.to_string_lossy().into_owned());
                }
            }
        }

        // Lista de paths da pasta ./dependencies
        let entries: IntoIter = WalkDir::new("./dependencies").into_iter();
        for entry in entries.filter_map(|e| e.ok()) {
            let path: &Path = entry.path();
            if path.is_dir() {
                if let Ok(relative_path) = path.strip_prefix("./dependencies") {
                    // Verifica se o caminho contém a pasta "Source"
                    let parts: Vec<&str> =
                        relative_path.iter().map(|p| p.to_str().unwrap()).collect();
                    if parts.len() > 1 && parts.contains(&"Source") {
                        dependency_paths.insert(format!(
                            ".\\dependencies\\{}",
                            relative_path.to_string_lossy()
                        ));
                    }
                }
            }
        }
        // Converte o HashSet para um Vec
        let dependency_paths: Vec<String> = dependency_paths.into_iter().collect();

        (dproj_files, dependency_paths)
    }
}
