use crate::error::{CliError, Result};
use regex::Regex;
use std::path::Path;

/// Palavras reservadas do Delphi que não podem ser usadas como nomes de módulos
const DELPHI_RESERVED_WORDS: &[&str] = &[
    "and", "array", "as", "asm", "begin", "case", "class", "const", "constructor",
    "destructor", "div", "do", "downto", "else", "end", "except", "exports",
    "file", "finalization", "finally", "for", "function", "goto", "if",
    "implementation", "in", "inherited", "initialization", "inline", "interface",
    "is", "label", "library", "mod", "nil", "not", "object", "of", "or",
    "out", "packed", "procedure", "program", "property", "raise", "record",
    "repeat", "resourcestring", "set", "shl", "shr", "string", "then", "threadvar",
    "to", "try", "type", "unit", "until", "uses", "var", "while", "with", "xor",
    "package", "boolean", "integer", "real", "char", "byte", "word", "longint",
    "shortint", "cardinal", "int64", "single", "double", "extended", "currency",
    "variant", "olevariant", "pointer", "pchar", "pwidechar", "ansistring",
    "widestring", "unicodestring", "rawbytestring", "utf8string", "ansichar",
    "widechar", "text", "absolute", "assembler", "cdecl", "dynamic", "export",
    "external", "far", "forward", "interrupt", "near", "override", "pascal",
    "private", "protected", "public", "published", "register", "reintroduce",
    "resident", "safecall", "stdcall", "static", "virtual", "abstract",
    "automated", "dispid", "readonly", "writeonly", "stored", "default",
    "nodefault", "index", "read", "write", "add", "remove", "implements",
    "name", "message", "contains", "requires", "deprecated", "platform",
    "reference", "helper", "sealed", "strict", "final", "operator", "overload",
    "dispinterface", "guid", "varargs", "unsafe", "true", "false", "at", "on",
    "raise"
];

/// Valida se um nome de módulo é válido
pub fn validate_module_name(name: &str) -> Result<()> {
    // Verifica se não está vazio
    if name.trim().is_empty() {
        return Err(CliError::validation_error("Module name cannot be empty"));
    }

    // Verifica tamanho mínimo e máximo
    if name.len() < 1 {
        return Err(CliError::validation_error("Module name must have at least 1 character"));
    }

    if name.len() > 50 {
        return Err(CliError::validation_error("Module name cannot have more than 50 characters"));
    }

    // Verifica se contém apenas caracteres válidos (letras, números, underscore)
    let valid_chars = Regex::new(r"^[a-zA-Z][a-zA-Z0-9_]*$")?;
    if !valid_chars.is_match(name) {
        return Err(CliError::invalid_module_name(name));
    }

    // Verifica se não é uma palavra reservada
    let name_lower = name.to_lowercase();
    if DELPHI_RESERVED_WORDS.contains(&name_lower.as_str()) {
        return Err(CliError::validation_error(
            format!("'{}' is a Delphi reserved word", name)
        ));
    }

    // Verifica se não começa com número
    if name.chars().next().unwrap().is_numeric() {
        return Err(CliError::validation_error("Module name cannot start with a number"));
    }

    Ok(())
}

/// Valida se um caminho de projeto é válido
pub fn validate_project_path(path: &str) -> Result<()> {
    if path.trim().is_empty() {
        return Err(CliError::validation_error("Project path cannot be empty"));
    }

    // Verifica se o caminho começa com ./
    if !path.starts_with("./") {
        return Err(CliError::validation_error(
            "Path must start with './'. Example: './my_project'"
        ));
    }

    Ok(())
}

/// Valida se um nome de projeto é válido
pub fn validate_project_name(name: &str) -> Result<()> {
    // Reutiliza a validação de módulo, mas com regras específicas para projeto
    validate_module_name(name)?;

    // Verifica se não contém espaços (projetos não devem ter espaços)
    if name.contains(' ') {
        return Err(CliError::validation_error(
            "Project name cannot contain spaces. Use underscore (_) or camelCase"
        ));
    }

    Ok(())
}

/// Valida URLs do Git (apenas GitHub HTTPS por enquanto)
pub fn validate_git_url(url: &str) -> Result<()> {
    if url.trim().is_empty() {
        return Err(CliError::validation_error("Git URL cannot be empty"));
    }

    // Verifica se é uma URL HTTPS do GitHub
    let github_regex = Regex::new(r"^https://github\.com/[\w\-\.]+/[\w\-\.]+(\.git)?/?$")?;
    if !github_regex.is_match(url) {
        return Err(CliError::validation_error(
            "URL must be a valid GitHub HTTPS URL (ex: https://github.com/user/repo.git)"
        ));
    }

    Ok(())
}

/// Verifica se um arquivo já existe e se deve ser sobrescrito
pub fn check_file_overwrite(file_path: &Path, overwrite: bool) -> Result<()> {
    if file_path.exists() && !overwrite {
        return Err(CliError::file_already_exists(
            file_path.display().to_string()
        ));
    }
    Ok(())
}

/// Valida se o diretório é um projeto Nest4D válido
pub fn validate_nest4d_project<P: AsRef<std::path::Path>>(path: P) -> Result<()> {
    let nest4d_path = path.as_ref().join("nest4d.json");
    if !nest4d_path.exists() {
        return Err(CliError::ProjectNotFound);
    }
    Ok(())
}

/// Valida se o diretório atual é um projeto Nest4D válido
pub fn validate_current_nest4d_project() -> Result<()> {
    validate_nest4d_project(std::env::current_dir()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_module_name_valid() {
        assert!(validate_module_name("User").is_ok());
        assert!(validate_module_name("UserService").is_ok());
        assert!(validate_module_name("User_Service").is_ok());
        assert!(validate_module_name("User123").is_ok());
    }

    #[test]
    fn test_validate_module_name_invalid() {
        assert!(validate_module_name("").is_err());
        assert!(validate_module_name("123User").is_err());
        assert!(validate_module_name("User-Service").is_err());
        assert!(validate_module_name("User Service").is_err());
        assert!(validate_module_name("begin").is_err()); // palavra reservada
        assert!(validate_module_name("package").is_err()); // palavra reservada
    }

    #[test]
    fn test_validate_project_path() {
        assert!(validate_project_path("./meu_projeto").is_ok());
        assert!(validate_project_path("/absolute/path").is_err());
        assert!(validate_project_path("relative/path").is_err());
        assert!(validate_project_path("").is_err());
    }

    #[test]
    fn test_validate_git_url() {
        assert!(validate_git_url("https://github.com/user/repo.git").is_ok());
        assert!(validate_git_url("https://gitlab.com/user/repo.git").is_err());
        assert!(validate_git_url("not-a-url").is_err());
        assert!(validate_git_url("").is_err());
    }
}