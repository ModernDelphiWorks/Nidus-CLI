use assert_cmd::Command;
use tempfile::TempDir;

/// Testa o comando --version
#[test]
fn test_version_command() {
    let mut cmd = Command::cargo_bin("nest4d").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicates::str::contains("nest4d"));
}

/// Testa o comando --help
#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("nest4d").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Nest4D CLI"));
}

/// Testa a criação de um novo projeto
#[test]
fn test_new_project_command() {
    let temp_dir = TempDir::new().unwrap();
    let project_name = "test_project";

    let mut cmd = Command::cargo_bin("nest4d").unwrap();
    cmd.current_dir(&temp_dir)
        .arg("new")
        .arg("--project")
        .arg(project_name)
        .arg("--path")
        .arg("./")
        .assert()
        .success();

    // Verifica se os arquivos foram criados
    let project_path = temp_dir.path().join(project_name);
    assert!(project_path.exists());
    assert!(project_path.join(format!("{}.dpr", project_name)).exists());
    assert!(project_path.join("src").join("AppModule.pas").exists());
}

/// Testa a criação de um projeto com testes
#[test]
fn test_new_project_with_tests() {
    let temp_dir = TempDir::new().unwrap();
    let project_name = "test_project_with_tests";

    let mut cmd = Command::cargo_bin("nest4d").unwrap();
    cmd.current_dir(&temp_dir)
        .arg("new")
        .arg("--project")
        .arg(project_name)
        .arg("--path")
        .arg("./")
        .arg("--with-tests")
        .assert()
        .success();

    // Verifica se a pasta de testes foi criada
    let project_path = temp_dir.path().join(project_name);
    assert!(project_path.join("test").exists());
}

/// Testa comando inválido
#[test]
fn test_invalid_command() {
    let mut cmd = Command::cargo_bin("nest4d").unwrap();
    cmd.arg("invalid_command").assert().failure();
}

/// Testa geração de módulo sem projeto
#[test]
fn test_gen_module_without_project() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("nest4d").unwrap();
    cmd.current_dir(&temp_dir)
        .arg("gen")
        .arg("module")
        .arg("User")
        .assert()
        .failure()
        .stderr(predicates::str::contains("nest4d.json"));
}

/// Testa a validação de nome de projeto inválido
#[test]
fn test_invalid_project_name() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("nest4d").unwrap();
    cmd.current_dir(&temp_dir)
        .arg("new")
        .arg("--project")
        .arg("123invalid") // Nome inválido (começa com número)
        .arg("--path")
        .arg("./")
        .assert()
        .failure();
}

/// Testa a validação de caminho inválido
#[test]
fn test_invalid_project_path() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("nest4d").unwrap();
    cmd.current_dir(&temp_dir)
        .arg("new")
        .arg("--project")
        .arg("valid_project")
        .arg("--path")
        .arg("/absolute/path") // Caminho inválido
        .assert()
        .failure();
}
