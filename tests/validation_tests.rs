use nidus::core::core_utils::utils;
use nidus::validation::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_validate_module_name_valid() {
    assert!(validate_module_name("User").is_ok());
    assert!(validate_module_name("UserService").is_ok());
    assert!(validate_module_name("MyModule").is_ok());
    assert!(validate_module_name("A").is_ok());
}

#[test]
fn test_validate_module_name_invalid() {
    // Empty name
    assert!(validate_module_name("").is_err());

    // Name too long
    let long_name = "A".repeat(51);
    assert!(validate_module_name(&long_name).is_err());

    // Invalid characters
    assert!(validate_module_name("User-Module").is_err());
    assert!(validate_module_name("User Module").is_err());
    assert!(validate_module_name("User@Module").is_err());

    // Starts with a digit
    assert!(validate_module_name("1User").is_err());

    // Delphi reserved words
    assert!(validate_module_name("begin").is_err());
    assert!(validate_module_name("end").is_err());
    assert!(validate_module_name("if").is_err());
    assert!(validate_module_name("then").is_err());
    assert!(validate_module_name("else").is_err());
    assert!(validate_module_name("while").is_err());
    assert!(validate_module_name("for").is_err());
    assert!(validate_module_name("do").is_err());
    assert!(validate_module_name("repeat").is_err());
    assert!(validate_module_name("until").is_err());
    assert!(validate_module_name("case").is_err());
    assert!(validate_module_name("of").is_err());
    assert!(validate_module_name("try").is_err());
    assert!(validate_module_name("except").is_err());
    assert!(validate_module_name("finally").is_err());
    assert!(validate_module_name("var").is_err());
    assert!(validate_module_name("const").is_err());
    assert!(validate_module_name("type").is_err());
    assert!(validate_module_name("function").is_err());
    assert!(validate_module_name("procedure").is_err());
    assert!(validate_module_name("class").is_err());
    assert!(validate_module_name("interface").is_err());
    assert!(validate_module_name("implementation").is_err());
    assert!(validate_module_name("unit").is_err());
    assert!(validate_module_name("uses").is_err());
    assert!(validate_module_name("program").is_err());
    assert!(validate_module_name("library").is_err());
    assert!(validate_module_name("package").is_err());
    assert!(validate_module_name("property").is_err());
    assert!(validate_module_name("published").is_err());
    assert!(validate_module_name("private").is_err());
    assert!(validate_module_name("protected").is_err());
    assert!(validate_module_name("public").is_err());
    assert!(validate_module_name("inherited").is_err());
    assert!(validate_module_name("override").is_err());
    assert!(validate_module_name("virtual").is_err());
    assert!(validate_module_name("abstract").is_err());
    assert!(validate_module_name("dynamic").is_err());
    assert!(validate_module_name("static").is_err());
    assert!(validate_module_name("constructor").is_err());
    assert!(validate_module_name("destructor").is_err());
    assert!(validate_module_name("record").is_err());
    assert!(validate_module_name("set").is_err());
    assert!(validate_module_name("array").is_err());
    assert!(validate_module_name("string").is_err());
    assert!(validate_module_name("integer").is_err());
    assert!(validate_module_name("boolean").is_err());
    assert!(validate_module_name("real").is_err());
    assert!(validate_module_name("char").is_err());
    assert!(validate_module_name("byte").is_err());
    assert!(validate_module_name("word").is_err());
    assert!(validate_module_name("longint").is_err());
    assert!(validate_module_name("shortint").is_err());
    assert!(validate_module_name("cardinal").is_err());
    assert!(validate_module_name("int64").is_err());
    assert!(validate_module_name("single").is_err());
    assert!(validate_module_name("double").is_err());
    assert!(validate_module_name("extended").is_err());
    assert!(validate_module_name("currency").is_err());
    assert!(validate_module_name("variant").is_err());
    assert!(validate_module_name("olevariant").is_err());
    assert!(validate_module_name("pointer").is_err());
    assert!(validate_module_name("pchar").is_err());
    assert!(validate_module_name("pwidechar").is_err());
    assert!(validate_module_name("ansistring").is_err());
    assert!(validate_module_name("widestring").is_err());
    assert!(validate_module_name("unicodestring").is_err());
    assert!(validate_module_name("rawbytestring").is_err());
    assert!(validate_module_name("utf8string").is_err());
    assert!(validate_module_name("ansichar").is_err());
    assert!(validate_module_name("widechar").is_err());
    assert!(validate_module_name("file").is_err());
    assert!(validate_module_name("text").is_err());
    assert!(validate_module_name("object").is_err());
    assert!(validate_module_name("packed").is_err());
    assert!(validate_module_name("with").is_err());
    assert!(validate_module_name("goto").is_err());
    assert!(validate_module_name("label").is_err());
    assert!(validate_module_name("nil").is_err());
    assert!(validate_module_name("true").is_err());
    assert!(validate_module_name("false").is_err());
    assert!(validate_module_name("and").is_err());
    assert!(validate_module_name("or").is_err());
    assert!(validate_module_name("not").is_err());
    assert!(validate_module_name("xor").is_err());
    assert!(validate_module_name("shl").is_err());
    assert!(validate_module_name("shr").is_err());
    assert!(validate_module_name("div").is_err());
    assert!(validate_module_name("mod").is_err());
    assert!(validate_module_name("in").is_err());
    assert!(validate_module_name("is").is_err());
    assert!(validate_module_name("as").is_err());
    assert!(validate_module_name("raise").is_err());
    assert!(validate_module_name("on").is_err());
    assert!(validate_module_name("at").is_err());
    assert!(validate_module_name("out").is_err());
    assert!(validate_module_name("threadvar").is_err());
    assert!(validate_module_name("resourcestring").is_err());
    assert!(validate_module_name("exports").is_err());
    assert!(validate_module_name("inline").is_err());
    assert!(validate_module_name("unsafe").is_err());
    assert!(validate_module_name("varargs").is_err());
    assert!(validate_module_name("cdecl").is_err());
    assert!(validate_module_name("pascal").is_err());
    assert!(validate_module_name("register").is_err());
    assert!(validate_module_name("safecall").is_err());
    assert!(validate_module_name("stdcall").is_err());
    assert!(validate_module_name("export").is_err());
    assert!(validate_module_name("far").is_err());
    assert!(validate_module_name("near").is_err());
    assert!(validate_module_name("resident").is_err());
    assert!(validate_module_name("absolute").is_err());
    assert!(validate_module_name("assembler").is_err());
    assert!(validate_module_name("external").is_err());
    assert!(validate_module_name("forward").is_err());
    assert!(validate_module_name("interrupt").is_err());
    assert!(validate_module_name("asm").is_err());
    assert!(validate_module_name("automated").is_err());
    assert!(validate_module_name("dispid").is_err());
    assert!(validate_module_name("readonly").is_err());
    assert!(validate_module_name("writeonly").is_err());
    assert!(validate_module_name("stored").is_err());
    assert!(validate_module_name("default").is_err());
    assert!(validate_module_name("nodefault").is_err());
    assert!(validate_module_name("index").is_err());
    assert!(validate_module_name("read").is_err());
    assert!(validate_module_name("write").is_err());
    assert!(validate_module_name("add").is_err());
    assert!(validate_module_name("remove").is_err());
    assert!(validate_module_name("implements").is_err());
    assert!(validate_module_name("name").is_err());
    assert!(validate_module_name("message").is_err());
    assert!(validate_module_name("contains").is_err());
    assert!(validate_module_name("requires").is_err());
    assert!(validate_module_name("finalization").is_err());
    assert!(validate_module_name("initialization").is_err());
    assert!(validate_module_name("deprecated").is_err());
    assert!(validate_module_name("library").is_err());
    assert!(validate_module_name("platform").is_err());
    assert!(validate_module_name("reference").is_err());
    assert!(validate_module_name("helper").is_err());
    assert!(validate_module_name("sealed").is_err());
    assert!(validate_module_name("strict").is_err());
    assert!(validate_module_name("final").is_err());
    assert!(validate_module_name("operator").is_err());
    assert!(validate_module_name("reintroduce").is_err());
    assert!(validate_module_name("overload").is_err());
    assert!(validate_module_name("dispinterface").is_err());
    assert!(validate_module_name("guid").is_err());
}

#[test]
fn test_validate_project_path_valid() {
    assert!(validate_project_path("./").is_ok());
    assert!(validate_project_path("./subfolder").is_ok());
    assert!(validate_project_path("./sub/folder").is_ok());
}

#[test]
fn test_validate_project_path_invalid() {
    assert!(validate_project_path("/absolute/path").is_err());
    assert!(validate_project_path("C:\\absolute\\path").is_err());
    assert!(validate_project_path("relative/path").is_err());
    assert!(validate_project_path("").is_err());
}

#[test]
fn test_validate_project_name_valid() {
    assert!(validate_project_name("MyProject").is_ok());
    assert!(validate_project_name("Project123").is_ok());
    assert!(validate_project_name("A").is_ok());
}

#[test]
fn test_validate_project_name_invalid() {
    // Name with spaces
    assert!(validate_project_name("My Project").is_err());

    // Other invalid cases (inherited from validate_module_name)
    assert!(validate_project_name("").is_err());
    assert!(validate_project_name("123Project").is_err());
    assert!(validate_project_name("begin").is_err());
}

#[test]
fn test_validate_git_url_valid() {
    // GitHub HTTPS
    assert!(validate_git_url("https://github.com/user/repo.git").is_ok());
    assert!(validate_git_url("https://github.com/user/repo").is_ok());
    assert!(validate_git_url("https://github.com/user-name/repo-name.git").is_ok());
    // Other HTTPS hosts (GitLab, Gitea, self-hosted)
    assert!(validate_git_url("https://gitlab.com/user/repo.git").is_ok());
    assert!(validate_git_url("https://gitea.example.com/user/repo").is_ok());
    // SSH
    assert!(validate_git_url("git@github.com:user/repo.git").is_ok());
    assert!(validate_git_url("git@gitlab.com:org/project.git").is_ok());
}

#[test]
fn test_validate_git_url_invalid() {
    assert!(validate_git_url("http://github.com/user/repo.git").is_err()); // HTTP (not HTTPS)
    assert!(validate_git_url("invalid-url").is_err());
    assert!(validate_git_url("").is_err());
    assert!(validate_git_url("ftp://github.com/user/repo").is_err()); // wrong scheme
    assert!(validate_git_url("https://github.com/user").is_err()); // missing repo part
}

#[test]
fn test_check_file_overwrite() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    // File does not exist — must pass
    assert!(check_file_overwrite(&file_path, false).is_ok());
    assert!(check_file_overwrite(&file_path, true).is_ok());

    // Create the file
    fs::write(&file_path, "test content").unwrap();

    // File exists, overwrite = false — must fail
    assert!(check_file_overwrite(&file_path, false).is_err());

    // File exists, overwrite = true — must pass
    assert!(check_file_overwrite(&file_path, true).is_ok());
}

#[test]
fn test_extract_repo_name_with_git_suffix() {
    assert_eq!(
        utils::extract_repo_name("https://github.com/user/my-repo.git"),
        Some("my-repo".to_string())
    );
    assert_eq!(
        utils::extract_repo_name("https://github.com/ModernDelphiWorks/Nidus.git"),
        Some("Nidus".to_string())
    );
}

#[test]
fn test_extract_repo_name_without_git_suffix() {
    assert_eq!(
        utils::extract_repo_name("https://github.com/user/my-repo"),
        Some("my-repo".to_string())
    );
    assert_eq!(
        utils::extract_repo_name("https://github.com/user/another-repo/"),
        Some("another-repo".to_string())
    );
}

#[test]
fn test_extract_repo_name_edge_cases() {
    // Simple name with .git suffix
    assert_eq!(
        utils::extract_repo_name("repo.git"),
        Some("repo".to_string())
    );
    // Simple name without .git suffix
    assert_eq!(
        utils::extract_repo_name("repo"),
        Some("repo".to_string())
    );
    // Empty string
    assert_eq!(utils::extract_repo_name(""), None);
}

#[test]
fn test_validate_nidus_project() {
    let temp_dir = TempDir::new().unwrap();

    // Without nidus.json — must fail
    assert!(validate_nidus_project(temp_dir.path()).is_err());

    // Create nidus.json
    let nidus_path = temp_dir.path().join("nidus.json");
    fs::write(&nidus_path, "{}").unwrap();

    // With nidus.json — must pass
    assert!(validate_nidus_project(temp_dir.path()).is_ok());
}
