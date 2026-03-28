use crate::error::{CliError, Result};
use regex::Regex;
use std::path::Path;

/// Delphi reserved words that cannot be used as module or project names
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

/// Validates whether a module name is valid
pub fn validate_module_name(name: &str) -> Result<()> {
    // Ensure name is not empty
    if name.trim().is_empty() {
        return Err(CliError::validation_error("Module name cannot be empty"));
    }

    // Enforce maximum length
    if name.len() > 50 {
        return Err(CliError::validation_error("Module name cannot have more than 50 characters"));
    }

    // Validate that only allowed characters are used (letters, digits, underscore)
    let valid_chars = Regex::new(r"^[a-zA-Z][a-zA-Z0-9_]*$")?;
    if !valid_chars.is_match(name) {
        return Err(CliError::invalid_module_name(name));
    }

    // Reject Delphi reserved words
    let name_lower = name.to_lowercase();
    if DELPHI_RESERVED_WORDS.contains(&name_lower.as_str()) {
        return Err(CliError::validation_error(
            format!("'{}' is a Delphi reserved word", name)
        ));
    }

    // Ensure the name does not start with a digit
    if name.chars().next().expect("name is non-empty, checked above").is_numeric() {
        return Err(CliError::validation_error("Module name cannot start with a number"));
    }

    Ok(())
}

/// Validates whether a project path is valid
pub fn validate_project_path(path: &str) -> Result<()> {
    if path.trim().is_empty() {
        return Err(CliError::validation_error("Project path cannot be empty"));
    }

    // Require relative paths starting with ./
    if !path.starts_with("./") {
        return Err(CliError::validation_error(
            "Path must start with './'. Example: './my_project'"
        ));
    }

    Ok(())
}

/// Validates whether a project name is valid
pub fn validate_project_name(name: &str) -> Result<()> {
    // Reuse module name validation with project-specific rules
    validate_module_name(name)?;

    // Project names must not contain spaces
    if name.contains(' ') {
        return Err(CliError::validation_error(
            "Project name cannot contain spaces. Use underscore (_) or camelCase"
        ));
    }

    Ok(())
}

/// Validates Git URLs — accepts any HTTPS or SSH git URL
pub fn validate_git_url(url: &str) -> Result<()> {
    if url.trim().is_empty() {
        return Err(CliError::validation_error("Git URL cannot be empty"));
    }

    // HTTPS: https://<host>/<user>/<repo>[.git][/]
    let https_re = Regex::new(r"^https://[^/]+/[\w\-\.]+/[\w\-\.]+(\.git)?/?$")?;
    // SSH:   git@<host>:<user>/<repo>[.git]
    let ssh_re = Regex::new(r"^git@[^:]+:[\w\-\.]+/[\w\-\.]+(\.git)?$")?;

    if !https_re.is_match(url) && !ssh_re.is_match(url) {
        return Err(CliError::validation_error(
            "URL must be a valid HTTPS (https://host/user/repo) or SSH (git@host:user/repo) git URL"
        ));
    }

    Ok(())
}

/// Checks whether a file already exists and whether it should be overwritten
pub fn check_file_overwrite(file_path: &Path, overwrite: bool) -> Result<()> {
    if file_path.exists() && !overwrite {
        return Err(CliError::file_already_exists(
            file_path.display().to_string()
        ));
    }
    Ok(())
}

/// Validates whether a directory contains a valid Nidus project
pub fn validate_nidus_project<P: AsRef<std::path::Path>>(path: P) -> Result<()> {
    let nidus_path = path.as_ref().join("nidus.json");
    if !nidus_path.exists() {
        return Err(CliError::ProjectNotFound);
    }
    Ok(())
}

/// Validates whether the current directory contains a valid Nidus project
pub fn validate_current_nidus_project() -> Result<()> {
    validate_nidus_project(std::env::current_dir()?)
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
        assert!(validate_module_name("begin").is_err()); // reserved word
        assert!(validate_module_name("package").is_err()); // reserved word
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
        assert!(validate_git_url("https://gitlab.com/user/repo.git").is_ok());
        assert!(validate_git_url("git@github.com:user/repo.git").is_ok());
        assert!(validate_git_url("http://github.com/user/repo.git").is_err()); // HTTP
        assert!(validate_git_url("not-a-url").is_err());
        assert!(validate_git_url("").is_err());
    }
}