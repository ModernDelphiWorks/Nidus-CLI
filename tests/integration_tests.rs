use assert_cmd::Command;
use git2::Repository;
use predicates::prelude::PredicateBooleanExt;
use std::path::Path;
use tempfile::TempDir;

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Creates a test project in the provided temporary directory.
fn setup_project(temp_dir: &TempDir, project_name: &str) {
    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(temp_dir)
        .arg("new")
        .arg(project_name)
        .arg("--path")
        .arg("./")
        .assert()
        .success();
}

/// Reads the content of the generated .dpr for a project.
fn read_dpr(project_dir: &std::path::Path, project_name: &str) -> String {
    std::fs::read_to_string(project_dir.join(format!("{}.dpr", project_name)))
        .expect(".dpr file should exist")
}

/// Runs `Nidus gen <subcommand> <name>` in the project directory.
fn gen(project_dir: &std::path::Path, subcommand: &str, name: &str) -> assert_cmd::assert::Assert {
    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(project_dir)
        .arg("gen")
        .arg(subcommand)
        .arg(name)
        .assert()
}

// ─── Tests: --version / --help ──────────────────────────────────────────────

#[test]
fn test_version_command() {
    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicates::str::contains("Nidus"));
}

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Nidus CLI"));
}

#[test]
fn test_invalid_command() {
    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.arg("invalid_command").assert().failure();
}

// ─── Tests: subcommands (info / templates / template list) ──────────────────

#[test]
fn test_templates_subcommand() {
    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.arg("templates")
        .assert()
        .success()
        .stdout(predicates::str::contains("controller.pas"))
        .stdout(predicates::str::contains("service.pas"));
}

#[test]
fn test_info_subcommand() {
    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.arg("info")
        .assert()
        .success()
        .stdout(predicates::str::contains("Nidus"));
}

#[test]
fn test_template_list_subcommand() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&temp_dir)
        .arg("template")
        .arg("list")
        .assert()
        .success();
}

// ─── Tests: `new` command ───────────────────────────────────────────────────

/// `new <name>` cria a pasta raiz do projeto.
#[test]
fn test_new_creates_project_directory() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyApp");
    assert!(temp_dir.path().join("MyApp").is_dir());
}

/// `new <name>` cria o arquivo .dpr com o nome correto do projeto.
#[test]
fn test_new_creates_dpr_file() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyApp");
    assert!(temp_dir.path().join("MyApp").join("MyApp.dpr").exists());
}

/// The .dpr content correctly substitutes `{{project}}` with the actual project name.
#[test]
fn test_new_dpr_contains_project_name() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyApp");
    let dpr = read_dpr(&temp_dir.path().join("MyApp"), "MyApp");
    assert!(dpr.contains("MyApp"), "dpr must contain the project name");
    assert!(!dpr.contains("{{project}}"), "template placeholder must be replaced");
}

/// `new` cria `src/AppModule.pas`.
#[test]
fn test_new_creates_appmodule() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyApp");
    assert!(temp_dir.path().join("MyApp").join("src").join("AppModule.pas").exists());
}

/// `new` cria a pasta `src/modules/`.
#[test]
fn test_new_creates_src_modules_dir() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyApp");
    assert!(temp_dir.path().join("MyApp").join("src").join("modules").is_dir());
}

/// `new --with-tests` também cria a pasta `test/`.
#[test]
fn test_new_with_tests_creates_test_dir() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&temp_dir)
        .arg("new")
        .arg("MyApp")
        .arg("--path")
        .arg("./")
        .arg("--with-tests")
        .assert()
        .success();
    assert!(temp_dir.path().join("MyApp").join("test").is_dir());
}

/// An invalid project name (starts with a digit) must fail.
#[test]
fn test_invalid_project_name() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&temp_dir)
        .arg("new")
        .arg("123invalid")
        .arg("--path")
        .arg("./")
        .assert()
        .failure();
}

/// An absolute path must fail (only relative paths are allowed).
#[test]
fn test_invalid_project_path() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&temp_dir)
        .arg("new")
        .arg("valid_project")
        .arg("--path")
        .arg("/absolute/path")
        .assert()
        .failure();
}

// ─── Tests: `gen` without a project (.dpr absent) ───────────────────────────

/// `gen module` without a .dpr must fail with a message mentioning .dpr.
#[test]
fn test_gen_module_without_project() {
    let temp_dir = TempDir::new().unwrap();
    gen(temp_dir.path(), "module", "User")
        .failure()
        .stderr(predicates::str::contains(".dpr"));
}

/// `gen controller` without a .dpr must fail.
#[test]
fn test_gen_controller_without_project() {
    let temp_dir = TempDir::new().unwrap();
    gen(temp_dir.path(), "controller", "User")
        .failure()
        .stderr(predicates::str::contains(".dpr"));
}

/// `gen service` without a .dpr must fail.
#[test]
fn test_gen_service_without_project() {
    let temp_dir = TempDir::new().unwrap();
    gen(temp_dir.path(), "service", "User")
        .failure()
        .stderr(predicates::str::contains(".dpr"));
}

/// `gen repository` without a .dpr must fail.
#[test]
fn test_gen_repository_without_project() {
    let temp_dir = TempDir::new().unwrap();
    gen(temp_dir.path(), "repository", "User")
        .failure()
        .stderr(predicates::str::contains(".dpr"));
}

/// `gen interface` without a .dpr must fail.
#[test]
fn test_gen_interface_without_project() {
    let temp_dir = TempDir::new().unwrap();
    gen(temp_dir.path(), "interface", "User")
        .failure()
        .stderr(predicates::str::contains(".dpr"));
}

/// `gen infra` without a .dpr must fail.
#[test]
fn test_gen_infra_without_project() {
    let temp_dir = TempDir::new().unwrap();
    gen(temp_dir.path(), "infra", "User")
        .failure()
        .stderr(predicates::str::contains(".dpr"));
}

/// `gen handler` without a .dpr must fail.
#[test]
fn test_gen_handler_without_project() {
    let temp_dir = TempDir::new().unwrap();
    gen(temp_dir.path(), "handler", "User")
        .failure()
        .stderr(predicates::str::contains(".dpr"));
}

/// `gen scaffold` without a .dpr must fail.
#[test]
fn test_gen_scaffold_without_project() {
    let temp_dir = TempDir::new().unwrap();
    gen(temp_dir.path(), "scaffold", "User")
        .failure()
        .stderr(predicates::str::contains(".dpr"));
}

/// `gen all` without a .dpr must fail.
#[test]
fn test_gen_all_without_project() {
    let temp_dir = TempDir::new().unwrap();
    gen(temp_dir.path(), "all", "User")
        .failure()
        .stderr(predicates::str::contains(".dpr"));
}

// ─── Tests: `gen module` ────────────────────────────────────────────────────

/// `gen module User` creates UserModule.pas and UserHandler.pas.
#[test]
fn test_gen_module_creates_module_and_handler_files() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    gen(&project_dir, "module", "User").success();

    let module_dir = project_dir.join("src").join("modules").join("user");
    assert!(module_dir.join("UserModule.pas").exists(), "UserModule.pas must be created");
    assert!(module_dir.join("UserHandler.pas").exists(), "UserHandler.pas must be created");
}

/// `gen module` does not generate controller/service/repository/interface/infra.
#[test]
fn test_gen_module_does_not_create_extra_files() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    gen(&project_dir, "module", "User").success();

    let module_dir = project_dir.join("src").join("modules").join("user");
    assert!(!module_dir.join("UserController.pas").exists());
    assert!(!module_dir.join("UserService.pas").exists());
    assert!(!module_dir.join("UserRepository.pas").exists());
}

/// After `gen module`, UserModule and UserHandler must be present in the .dpr.
#[test]
fn test_gen_module_adds_units_to_dpr() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    gen(&project_dir, "module", "User").success();

    let dpr = read_dpr(&project_dir, "MyProject");
    assert!(dpr.contains("UserModule"), ".dpr must contain UserModule");
    assert!(dpr.contains("UserHandler"), ".dpr must contain UserHandler");
}

/// Generated file content correctly substitutes `{{mod}}` with the actual module name.
#[test]
fn test_gen_module_files_have_correct_content() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    gen(&project_dir, "module", "User").success();

    let module_dir = project_dir.join("src").join("modules").join("user");
    let module_content = std::fs::read_to_string(module_dir.join("UserModule.pas")).unwrap();
    assert!(!module_content.contains("{{mod}}"), "template placeholder must be replaced in UserModule.pas");
    assert!(module_content.contains("User"), "generated file must reference module name");
}

// ─── Tests: `gen controller` ────────────────────────────────────────────────

/// `gen controller Product` cria ProductController.pas.
#[test]
fn test_gen_controller_creates_file() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    gen(&project_dir, "controller", "Product").success();

    let module_dir = project_dir.join("src").join("modules").join("product");
    assert!(module_dir.join("ProductController.pas").exists());
}

/// After `gen controller`, ProductController must be present in the .dpr.
#[test]
fn test_gen_controller_adds_unit_to_dpr() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    gen(&project_dir, "controller", "Product").success();

    let dpr = read_dpr(&project_dir, "MyProject");
    assert!(dpr.contains("ProductController"), ".dpr must contain ProductController");
}

// ─── Tests: `gen service` ───────────────────────────────────────────────────

/// `gen service Order` cria OrderService.pas.
#[test]
fn test_gen_service_creates_file() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    gen(&project_dir, "service", "Order").success();

    let module_dir = project_dir.join("src").join("modules").join("order");
    assert!(module_dir.join("OrderService.pas").exists());
}

/// After `gen service`, OrderService must be present in the .dpr.
#[test]
fn test_gen_service_adds_unit_to_dpr() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    gen(&project_dir, "service", "Order").success();

    let dpr = read_dpr(&project_dir, "MyProject");
    assert!(dpr.contains("OrderService"), ".dpr must contain OrderService");
}

// ─── Tests: `gen repository` ────────────────────────────────────────────────

/// `gen repository Customer` cria CustomerRepository.pas.
#[test]
fn test_gen_repository_creates_file() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    gen(&project_dir, "repository", "Customer").success();

    let module_dir = project_dir.join("src").join("modules").join("customer");
    assert!(module_dir.join("CustomerRepository.pas").exists());
}

/// After `gen repository`, CustomerRepository must be present in the .dpr.
#[test]
fn test_gen_repository_adds_unit_to_dpr() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    gen(&project_dir, "repository", "Customer").success();

    let dpr = read_dpr(&project_dir, "MyProject");
    assert!(dpr.contains("CustomerRepository"), ".dpr must contain CustomerRepository");
}

// ─── Tests: `gen interface` ─────────────────────────────────────────────────

/// `gen interface Payment` cria PaymentInterface.pas.
#[test]
fn test_gen_interface_creates_file() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    gen(&project_dir, "interface", "Payment").success();

    let module_dir = project_dir.join("src").join("modules").join("payment");
    assert!(module_dir.join("PaymentInterface.pas").exists());
}

/// After `gen interface`, PaymentInterface must be present in the .dpr.
#[test]
fn test_gen_interface_adds_unit_to_dpr() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    gen(&project_dir, "interface", "Payment").success();

    let dpr = read_dpr(&project_dir, "MyProject");
    assert!(dpr.contains("PaymentInterface"), ".dpr must contain PaymentInterface");
}

// ─── Tests: `gen infra` ─────────────────────────────────────────────────────

/// `gen infra Auth` cria AuthInfra.pas.
#[test]
fn test_gen_infra_creates_file() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    gen(&project_dir, "infra", "Auth").success();

    let module_dir = project_dir.join("src").join("modules").join("auth");
    assert!(module_dir.join("AuthInfra.pas").exists());
}

/// After `gen infra`, AuthInfra must be present in the .dpr.
#[test]
fn test_gen_infra_adds_unit_to_dpr() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    gen(&project_dir, "infra", "Auth").success();

    let dpr = read_dpr(&project_dir, "MyProject");
    assert!(dpr.contains("AuthInfra"), ".dpr must contain AuthInfra");
}

// ─── Tests: `gen handler` ───────────────────────────────────────────────────

/// `gen handler Notification` cria NotificationHandler.pas.
#[test]
fn test_gen_handler_creates_file() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    gen(&project_dir, "handler", "Notification").success();

    let module_dir = project_dir.join("src").join("modules").join("notification");
    assert!(module_dir.join("NotificationHandler.pas").exists());
}

/// After `gen handler`, NotificationHandler must be present in the .dpr.
#[test]
fn test_gen_handler_adds_unit_to_dpr() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    gen(&project_dir, "handler", "Notification").success();

    let dpr = read_dpr(&project_dir, "MyProject");
    assert!(dpr.contains("NotificationHandler"), ".dpr must contain NotificationHandler");
}

// ─── Tests: `gen scaffold` ──────────────────────────────────────────────────

/// `gen scaffold` cria todos os 7 arquivos do módulo completo.
#[test]
fn test_gen_scaffold_creates_all_seven_files() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    gen(&project_dir, "scaffold", "Invoice").success();

    let module_dir = project_dir.join("src").join("modules").join("invoice");
    assert!(module_dir.join("InvoiceModule.pas").exists(),     "InvoiceModule.pas");
    assert!(module_dir.join("InvoiceHandler.pas").exists(),    "InvoiceHandler.pas");
    assert!(module_dir.join("InvoiceController.pas").exists(), "InvoiceController.pas");
    assert!(module_dir.join("InvoiceService.pas").exists(),    "InvoiceService.pas");
    assert!(module_dir.join("InvoiceRepository.pas").exists(), "InvoiceRepository.pas");
    assert!(module_dir.join("InvoiceInterface.pas").exists(),  "InvoiceInterface.pas");
    assert!(module_dir.join("InvoiceInfra.pas").exists(),      "InvoiceInfra.pas");
}

/// After `gen scaffold`, all 7 units must be present in the .dpr.
#[test]
fn test_gen_scaffold_adds_all_units_to_dpr() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    gen(&project_dir, "scaffold", "Invoice").success();

    let dpr = read_dpr(&project_dir, "MyProject");
    assert!(dpr.contains("InvoiceModule"),     ".dpr must contain InvoiceModule");
    assert!(dpr.contains("InvoiceHandler"),    ".dpr must contain InvoiceHandler");
    assert!(dpr.contains("InvoiceController"), ".dpr must contain InvoiceController");
    assert!(dpr.contains("InvoiceService"),    ".dpr must contain InvoiceService");
    assert!(dpr.contains("InvoiceRepository"), ".dpr must contain InvoiceRepository");
    assert!(dpr.contains("InvoiceInterface"),  ".dpr must contain InvoiceInterface");
    assert!(dpr.contains("InvoiceInfra"),      ".dpr must contain InvoiceInfra");
}

// ─── Tests: `gen all` ───────────────────────────────────────────────────────

/// `gen all` cria todos os 7 arquivos do módulo completo.
#[test]
fn test_gen_all_creates_all_seven_files() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    gen(&project_dir, "all", "Report").success();

    let module_dir = project_dir.join("src").join("modules").join("report");
    assert!(module_dir.join("ReportModule.pas").exists(),     "ReportModule.pas");
    assert!(module_dir.join("ReportHandler.pas").exists(),    "ReportHandler.pas");
    assert!(module_dir.join("ReportController.pas").exists(), "ReportController.pas");
    assert!(module_dir.join("ReportService.pas").exists(),    "ReportService.pas");
    assert!(module_dir.join("ReportRepository.pas").exists(), "ReportRepository.pas");
    assert!(module_dir.join("ReportInterface.pas").exists(),  "ReportInterface.pas");
    assert!(module_dir.join("ReportInfra.pas").exists(),      "ReportInfra.pas");
}

/// After `gen all`, all 7 units must be present in the .dpr.
#[test]
fn test_gen_all_adds_all_units_to_dpr() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    gen(&project_dir, "all", "Report").success();

    let dpr = read_dpr(&project_dir, "MyProject");
    assert!(dpr.contains("ReportModule"),     ".dpr must contain ReportModule");
    assert!(dpr.contains("ReportHandler"),    ".dpr must contain ReportHandler");
    assert!(dpr.contains("ReportController"), ".dpr must contain ReportController");
    assert!(dpr.contains("ReportService"),    ".dpr must contain ReportService");
    assert!(dpr.contains("ReportRepository"), ".dpr must contain ReportRepository");
    assert!(dpr.contains("ReportInterface"),  ".dpr must contain ReportInterface");
    assert!(dpr.contains("ReportInfra"),      ".dpr must contain ReportInfra");
}

// ─── Tests: multiple `gen` calls accumulate in the same .dpr ────────────────

/// Successive generations of different modules accumulate units in the .dpr without duplicates.
#[test]
fn test_multiple_gen_accumulate_in_dpr() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    gen(&project_dir, "module", "User").success();
    gen(&project_dir, "module", "Product").success();

    let dpr = read_dpr(&project_dir, "MyProject");
    assert!(dpr.contains("UserModule"),    ".dpr must contain UserModule");
    assert!(dpr.contains("UserHandler"),   ".dpr must contain UserHandler");
    assert!(dpr.contains("ProductModule"), ".dpr must contain ProductModule");
    assert!(dpr.contains("ProductHandler"),".dpr must contain ProductHandler");
}

/// Repeating the same `gen module` does not duplicate entries in the .dpr.
/// The entry `UserModule in 'path'` must appear exactly once.
#[test]
fn test_gen_module_repeated_no_dpr_duplicates() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    gen(&project_dir, "module", "User").success();
    // Second generation — must skip units already present in the .dpr
    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&project_dir)
        .arg("gen")
        .arg("module")
        .arg("User")
        .arg("--overwrite")
        .assert()
        .success();

    let dpr = read_dpr(&project_dir, "MyProject");
    // "UserModule in '" is the unit declaration in the uses block — must appear exactly once
    let count = dpr.matches("UserModule in '").count();
    assert_eq!(count, 1, "UserModule declaration must appear exactly once in .dpr (no duplicates)");
}

// ─── Tests: `sync` command (add-paths → .dproj) ─────────────────────────────

/// Creates a minimal .dproj compatible with Delphi/MSBuild.
fn make_dproj(dir: &std::path::Path, name: &str) {
    let content = r#"<Project xmlns="http://schemas.microsoft.com/developer/msbuild/2003">
  <PropertyGroup Condition="'$(Base)'!=''">
  </PropertyGroup>
</Project>"#;
    std::fs::write(dir.join(format!("{}.dproj", name)), content).unwrap();
}

/// Creates a simulated dependency structure (no git clone).
fn make_dependency_src(dir: &std::path::Path, package: &str, src_dir: &str) {
    std::fs::create_dir_all(dir.join("dependencies").join(package).join(src_dir)).unwrap();
}

/// Extracts the `DCC_UnitSearchPath` node value and returns each `;`-separated item.
fn extract_search_paths(dproj: &str) -> Vec<String> {
    let tag_open  = "<DCC_UnitSearchPath>";
    let tag_close = "</DCC_UnitSearchPath>";
    if let Some(start) = dproj.find(tag_open) {
        let after = &dproj[start + tag_open.len()..];
        if let Some(end) = after.find(tag_close) {
            return after[..end]
                .split(';')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect();
        }
    }
    vec![]
}

/// Verifies that all paths (except the macro) are relative, never absolute.
fn assert_all_paths_relative(dproj: &str) {
    let paths = extract_search_paths(dproj);
    assert!(!paths.is_empty(), "DCC_UnitSearchPath must have entries");
    for path in &paths {
        if path.contains("$(") { continue; }  // macros are allowed
        assert!(
            path.starts_with(r".\") || path.starts_with(r"..\"),
            "path must start with .\\ or ..\\ (relative), got: {:?}",
            path
        );
        // Must never contain an absolute system path
        assert!(
            !path.starts_with('/') && !path.contains(":/"),
            "path must not be absolute, got: {:?}",
            path
        );
    }
}

/// `sync` adds the dependency path to the .dproj.
#[test]
fn test_sync_adds_dependency_path_to_dproj() {
    let temp_dir = TempDir::new().unwrap();
    make_dproj(temp_dir.path(), "MyApp");
    make_dependency_src(temp_dir.path(), "Horse", "src");

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&temp_dir)
        .arg("sync")
        .assert()
        .success();

    let dproj = std::fs::read_to_string(temp_dir.path().join("MyApp.dproj")).unwrap();
    assert!(dproj.contains("Horse"), ".dproj must contain Horse path after sync");
}

/// The path written to the .dproj is RELATIVE (not absolute) — portable project.
#[test]
fn test_sync_paths_are_relative_not_absolute() {
    let temp_dir = TempDir::new().unwrap();
    make_dproj(temp_dir.path(), "MyApp");
    make_dependency_src(temp_dir.path(), "Horse", "src");
    make_dependency_src(temp_dir.path(), "Nidus", "Source");

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&temp_dir)
        .arg("sync")
        .assert()
        .success();

    let dproj = std::fs::read_to_string(temp_dir.path().join("MyApp.dproj")).unwrap();
    assert_all_paths_relative(&dproj);
}

/// Paths use the `\` Windows-style separator (compatible with Delphi/MSBuild).
#[test]
fn test_sync_paths_use_windows_separator() {
    let temp_dir = TempDir::new().unwrap();
    make_dproj(temp_dir.path(), "MyApp");
    make_dependency_src(temp_dir.path(), "Horse", "src");

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&temp_dir)
        .arg("sync")
        .assert()
        .success();

    let dproj = std::fs::read_to_string(temp_dir.path().join("MyApp.dproj")).unwrap();
    let paths = extract_search_paths(&dproj);
    let horse_path = paths.iter()
        .find(|p| p.contains("Horse"))
        .expect("Horse path must be present");

    assert!(
        horse_path.contains('\\'),
        "path must use \\ separator for Delphi, got: {:?}",
        horse_path
    );
    assert_eq!(
        horse_path, r".\dependencies\Horse\src",
        "expected exact relative path"
    );
}

/// `sync` adds multiple dependencies at once.
#[test]
fn test_sync_adds_multiple_dependencies_to_dproj() {
    let temp_dir = TempDir::new().unwrap();
    make_dproj(temp_dir.path(), "MyApp");
    make_dependency_src(temp_dir.path(), "Horse", "src");
    make_dependency_src(temp_dir.path(), "Nidus", "Source");
    make_dependency_src(temp_dir.path(), "InjectContainer", "src");

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&temp_dir)
        .arg("sync")
        .assert()
        .success();

    let dproj = std::fs::read_to_string(temp_dir.path().join("MyApp.dproj")).unwrap();
    assert!(dproj.contains("Horse"),           "Horse must be in .dproj");
    assert!(dproj.contains("Nidus"),           "Nidus must be in .dproj");
    assert!(dproj.contains("InjectContainer"), "InjectContainer must be in .dproj");
}

/// After `sync`, the macro `$(DCC_UnitSearchPath)` must appear at the end.
#[test]
fn test_sync_macro_is_last_in_dproj() {
    let temp_dir = TempDir::new().unwrap();
    make_dproj(temp_dir.path(), "MyApp");
    make_dependency_src(temp_dir.path(), "Horse", "src");

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&temp_dir)
        .arg("sync")
        .assert()
        .success();

    let dproj = std::fs::read_to_string(temp_dir.path().join("MyApp.dproj")).unwrap();
    let macro_pos = dproj.find("$(DCC_UnitSearchPath)").expect("macro must be present");
    let horse_pos = dproj.find("Horse").expect("Horse must be present");
    assert!(horse_pos < macro_pos, "$(DCC_UnitSearchPath) must come after all paths");
}

/// Running `sync` twice does not duplicate entries in the .dproj (idempotent).
#[test]
fn test_sync_idempotent_no_duplicates() {
    let temp_dir = TempDir::new().unwrap();
    make_dproj(temp_dir.path(), "MyApp");
    make_dependency_src(temp_dir.path(), "Horse", "src");

    for _ in 0..2 {
        let mut cmd = Command::cargo_bin("Nidus").unwrap();
        cmd.current_dir(&temp_dir)
            .arg("sync")
            .assert()
            .success();
    }

    let dproj = std::fs::read_to_string(temp_dir.path().join("MyApp.dproj")).unwrap();
    assert_eq!(
        dproj.matches("Horse").count(),
        1,
        "Horse must appear exactly once after two syncs"
    );
}

/// `sync` without a .dproj must fail with an error message.
#[test]
fn test_sync_fails_without_dproj() {
    let temp_dir = TempDir::new().unwrap();
    make_dependency_src(temp_dir.path(), "Horse", "src");

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&temp_dir)
        .arg("sync")
        .assert()
        .failure();
}

/// `sync` without a `dependencies` folder (or without `src`/`Source` inside) must fail.
#[test]
fn test_sync_fails_without_dependency_src_dirs() {
    let temp_dir = TempDir::new().unwrap();
    make_dproj(temp_dir.path(), "MyApp");
    // dependencies exists but without src/Source
    std::fs::create_dir_all(temp_dir.path().join("dependencies/Horse/bin")).unwrap();

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&temp_dir)
        .arg("sync")
        .assert()
        .failure();
}

/// `sync` with a dependency that has nested subfolders adds ALL paths to the .dproj.
#[test]
fn test_sync_adds_all_nested_subdirs_to_dproj() {
    let temp_dir = TempDir::new().unwrap();
    make_dproj(temp_dir.path(), "MyApp");

    // Real Horse structure: src/ + src/Pipes/ + src/Core/ + src/Core/Utils/
    std::fs::create_dir_all(temp_dir.path().join("dependencies/Horse/src/Pipes")).unwrap();
    std::fs::create_dir_all(temp_dir.path().join("dependencies/Horse/src/Core/Utils")).unwrap();

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&temp_dir)
        .arg("sync")
        .assert()
        .success();

    let dproj = std::fs::read_to_string(temp_dir.path().join("MyApp.dproj")).unwrap();
    // All nesting levels must be present in the .dproj
    assert!(dproj.contains("Horse"),  "Horse/src must be present");
    assert!(dproj.contains("Pipes"),  "Horse/src/Pipes must be present");
    assert!(dproj.contains("Core"),   "Horse/src/Core must be present");
    assert!(dproj.contains("Utils"),  "Horse/src/Core/Utils must be present");
}

/// `add-paths` é alias visível de `sync` e produz o mesmo resultado.
#[test]
fn test_add_paths_alias_works_same_as_sync() {
    let temp_dir = TempDir::new().unwrap();
    make_dproj(temp_dir.path(), "MyApp");
    make_dependency_src(temp_dir.path(), "Horse", "src");

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&temp_dir)
        .arg("add-paths")
        .assert()
        .success();

    let dproj = std::fs::read_to_string(temp_dir.path().join("MyApp.dproj")).unwrap();
    assert!(dproj.contains("Horse"), ".dproj must contain Horse path after add-paths");
}

// ─── Tests: `install` command (git clone) ────────────────────────────────────

/// Cria um repo local com um commit inicial e retorna a URL file://.
fn create_local_repo(dir: &Path) -> String {
    let repo = Repository::init(dir).unwrap();
    let sig  = git2::Signature::now("Test", "test@nidus.dev").unwrap();
    let mut index = repo.index().unwrap();

    std::fs::write(dir.join("README.md"), "# Test repo").unwrap();
    index.add_path(Path::new("README.md")).unwrap();
    index.write().unwrap();

    let tree_oid = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_oid).unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "initial commit", &tree, &[]).unwrap();

    format!("file://{}", dir.display())
}

/// Writes a nidus.json with download and dependencies pointing to local repos.
fn write_nidus_json(dir: &Path, mainsrc: &str, download_url: &str, extra_deps: &[(&str, &str)]) {
    let mut deps = extra_deps
        .iter()
        .map(|(url, branch)| format!("    \"{}\": \"{}\"", url, branch))
        .collect::<Vec<_>>()
        .join(",\n");
    if !deps.is_empty() { deps.push('\n'); }

    let content = format!(
        r#"{{
  "name": "TestProject",
  "description": "Test",
  "version": "main",
  "homepage": "http://localhost",
  "mainsrc": "{}",
  "projects": [],
  "download": "{}",
  "dependencies": {{
{}  }}
}}"#,
        mainsrc, download_url, deps
    );
    std::fs::write(dir.join("nidus.json"), content).unwrap();
}

/// `install` clona o repositório de download em `mainsrc/<repo_name>/`.
#[test]
fn test_install_clones_repo_into_dependencies() {
    let project_dir  = TempDir::new().unwrap();
    let upstream_dir = TempDir::new().unwrap();

    let repo_url = create_local_repo(upstream_dir.path());
    write_nidus_json(project_dir.path(), "./dependencies", &repo_url, &[]);

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&project_dir)
        .arg("install")
        .assert()
        .success();

    // Extrai o nome do repo da URL (último segmento do path)
    let repo_name = upstream_dir.path().file_name().unwrap().to_str().unwrap();
    let cloned = project_dir.path().join("dependencies").join(repo_name);
    assert!(cloned.exists(), "cloned directory must exist: {:?}", cloned);
    assert!(cloned.join("README.md").exists(), "repo content must be present");
}

/// `install` com múltiplas dependências clona todas.
#[test]
fn test_install_clones_multiple_dependencies() {
    let project_dir = TempDir::new().unwrap();
    let upstream_a  = TempDir::new().unwrap();
    let upstream_b  = TempDir::new().unwrap();

    let url_a = create_local_repo(upstream_a.path());
    let url_b = create_local_repo(upstream_b.path());

    // download = url_a; additional dependency = url_b (empty branch = use repo default)
    write_nidus_json(project_dir.path(), "./dependencies", &url_a, &[(&url_b, "")]);

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&project_dir)
        .arg("install")
        .assert()
        .success();

    let name_a = upstream_a.path().file_name().unwrap().to_str().unwrap();
    let name_b = upstream_b.path().file_name().unwrap().to_str().unwrap();
    assert!(project_dir.path().join("dependencies").join(name_a).exists(), "repo A must be cloned");
    assert!(project_dir.path().join("dependencies").join(name_b).exists(), "repo B must be cloned");
}

/// `install` pula repositório que já foi clonado (pasta existente).
#[test]
fn test_install_skips_existing_clone() {
    let project_dir  = TempDir::new().unwrap();
    let upstream_dir = TempDir::new().unwrap();

    let repo_url = create_local_repo(upstream_dir.path());
    write_nidus_json(project_dir.path(), "./dependencies", &repo_url, &[]);

    // First installation
    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&project_dir)
        .arg("install")
        .assert()
        .success();

    // Add a sentinel file to detect if the clone is overwritten
    let repo_name = upstream_dir.path().file_name().unwrap().to_str().unwrap();
    let cloned_dir = project_dir.path().join("dependencies").join(repo_name);
    let sentinel = cloned_dir.join("sentinel.txt");
    std::fs::write(&sentinel, "must survive second install").unwrap();

    // Second installation — must skip, not delete the folder
    let mut cmd2 = Command::cargo_bin("Nidus").unwrap();
    cmd2.current_dir(&project_dir)
        .arg("install")
        .assert()
        .success()
        .stdout(predicates::str::contains("already exists"));

    assert!(sentinel.exists(), "existing clone must not be overwritten");
}

/// `install` reporta sucesso no stdout.
#[test]
fn test_install_reports_cloned_successfully() {
    let project_dir  = TempDir::new().unwrap();
    let upstream_dir = TempDir::new().unwrap();

    let repo_url = create_local_repo(upstream_dir.path());
    write_nidus_json(project_dir.path(), "./dependencies", &repo_url, &[]);

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&project_dir)
        .arg("install")
        .assert()
        .success()
        .stdout(predicates::str::contains("cloned successfully").or(predicates::str::contains("Cloned")));
}

/// `install` with an invalid URL must fail and report the error in the summary.
#[test]
fn test_install_reports_failure_for_invalid_url() {
    let project_dir = TempDir::new().unwrap();
    write_nidus_json(
        project_dir.path(),
        "./dependencies",
        "file:///nonexistent/path/that/does/not/exist",
        &[],
    );

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&project_dir)
        .arg("install")
        .assert()
        .success() // o comando não faz process::exit para falhas individuais
        .stdout(predicates::str::contains("Failed").or(predicates::str::contains("❌")));
}

// ─── Tests: `install --add` (N1) ─────────────────────────────────────────────

/// `install --add` com URL inválida deve falhar com exit code 1.
#[test]
fn test_install_add_rejects_invalid_url() {
    let project_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&project_dir)
        .args(["install", "--add", "not-a-valid-url"])
        .assert()
        .failure();
}

/// `install --add` com URL fictícia: o clone falha e o rollback remove a URL do nidus.json.
#[test]
fn test_install_add_rollback_removes_url_on_failed_clone() {
    let project_dir = TempDir::new().unwrap();
    let new_dep = "https://github.com/fake-user/fake-repo-nonexistent-xyz.git";

    write_nidus_json(
        project_dir.path(),
        "./dependencies",
        "https://github.com/ModernDelphiWorks/Nidus.git",
        &[],
    );

    // O clone vai falhar — exit code é 0 (falhas individuais são impressas, não terminam o processo)
    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&project_dir)
        .args(["install", "--add", new_dep])
        .assert()
        .success();

    let nidus_json =
        std::fs::read_to_string(project_dir.path().join("nidus.json")).unwrap();
    assert!(
        !nidus_json.contains(new_dep),
        "Rollback deve ter removido a URL do nidus.json após clone falhar, got:\n{}",
        nidus_json
    );
}

/// `install --add` com URL duplicada deve falhar com mensagem de erro.
#[test]
fn test_install_add_rejects_duplicate_dependency() {
    let project_dir = TempDir::new().unwrap();
    let dep_url = "https://github.com/fake-user/fake-repo.git";

    // Escreve nidus.json com a dependência já presente
    write_nidus_json(
        project_dir.path(),
        "./dependencies",
        "https://github.com/ModernDelphiWorks/Nidus.git",
        &[(dep_url, "")],
    );

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&project_dir)
        .args(["install", "--add", dep_url])
        .assert()
        .failure()
        .stderr(predicates::str::contains("already in nidus.json"));
}

/// `install --add` sem nidus.json: o arquivo é criado automaticamente.
/// O clone vai falhar (URL fictícia) e o rollback remove a URL, mas o arquivo continua existindo.
#[test]
fn test_install_add_creates_nidus_json_when_missing() {
    let project_dir = TempDir::new().unwrap();
    let new_dep = "https://github.com/fake-user/new-lib-xyz-nonexistent.git";

    // Não há nidus.json — deve ser criado automaticamente
    assert!(!project_dir.path().join("nidus.json").exists());

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&project_dir)
        .args(["install", "--add", new_dep])
        .assert()
        .success();

    // nidus.json deve ter sido criado (mesmo após rollback da URL)
    assert!(
        project_dir.path().join("nidus.json").exists(),
        "nidus.json deve ser criado quando ausente"
    );
}

/// `install --add` com `--branch` e URL fictícia: rollback remove URL+branch do nidus.json.
#[test]
fn test_install_add_with_branch_rollback_on_failed_clone() {
    let project_dir = TempDir::new().unwrap();
    let new_dep = "https://github.com/fake-user/fake-repo-xyz-nonexistent.git";

    write_nidus_json(
        project_dir.path(),
        "./dependencies",
        "https://github.com/ModernDelphiWorks/Nidus.git",
        &[],
    );

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&project_dir)
        .args(["install", "--add", new_dep, "--branch", "develop"])
        .assert()
        .success();

    let nidus_json =
        std::fs::read_to_string(project_dir.path().join("nidus.json")).unwrap();
    assert!(
        !nidus_json.contains(new_dep),
        "Rollback deve ter removido a URL do nidus.json, got:\n{}",
        nidus_json
    );
}

// ─── Tests: `gen --template` (N2) ─────────────────────────────────────────────

/// `gen module --template nome-inexistente` deve usar fallback built-in e gerar os arquivos.
#[test]
fn test_gen_module_with_nonexistent_template_falls_back_to_builtin() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&project_dir)
        .args(["gen", "module", "User", "--template", "nonexistent-template"])
        .assert()
        .success();

    // Arquivos devem ter sido criados via fallback built-in
    let module_dir = project_dir.join("src").join("modules").join("user");
    assert!(module_dir.join("UserModule.pas").exists(), "UserModule.pas deve existir");
    assert!(module_dir.join("UserHandler.pas").exists(), "UserHandler.pas deve existir");
}

/// `gen module --template nome-inexistente` deve emitir aviso no stderr para cada componente.
#[test]
fn test_gen_module_with_nonexistent_template_emits_warning() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&project_dir)
        .args(["gen", "module", "Order", "--template", "nonexistent-template"])
        .assert()
        .success()
        .stderr(predicates::str::contains("not found in template"));
}

/// `gen module --template` com template válido (built-in exportado) usa o template.
#[test]
fn test_gen_module_with_builtin_template_name_succeeds() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    // default-module existe como template built-in no TemplateManager
    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&project_dir)
        .args(["gen", "module", "Product", "--template", "default-module"])
        .assert()
        .success();

    let module_dir = project_dir.join("src").join("modules").join("product");
    assert!(module_dir.join("ProductModule.pas").exists());
    assert!(module_dir.join("ProductHandler.pas").exists());
}

// ─── Tests: `update` command (CLI end-to-end) ────────────────────────────────

/// Helper: clona um repo local e retorna o caminho da pasta clonada.
fn clone_repo_locally(upstream_url: &str, dest: &Path) -> Repository {
    Repository::clone(upstream_url, dest).unwrap()
}

/// Helper: adds a new commit to the upstream to simulate available updates.
fn push_new_commit(upstream_dir: &Path, message: &str) {
    let repo = Repository::open(upstream_dir).unwrap();
    let sig  = git2::Signature::now("Test", "test@nidus.dev").unwrap();
    let mut index = repo.index().unwrap();

    std::fs::write(upstream_dir.join("change.txt"), message).unwrap();
    index.add_path(Path::new("change.txt")).unwrap();
    index.write().unwrap();

    let tree_oid = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_oid).unwrap();
    let parent = repo.head().unwrap().peel_to_commit().unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[&parent]).unwrap();
}

/// `Nidus update` fast-forwards a dependency with new commits.
#[test]
fn test_update_fast_forwards_dependency() {
    let project_dir  = TempDir::new().unwrap();
    let upstream_dir = TempDir::new().unwrap();

    // Create upstream + local clone
    let repo_url = create_local_repo(upstream_dir.path());
    let repo_name = upstream_dir.path().file_name().unwrap().to_str().unwrap();
    let clone_dest = project_dir.path().join("dependencies").join(repo_name);
    std::fs::create_dir_all(&clone_dest).unwrap();
    clone_repo_locally(&repo_url, &clone_dest);

    // Add a new commit to the upstream
    push_new_commit(upstream_dir.path(), "update commit");

    // Configure nidus.json to point to the upstream
    write_nidus_json(project_dir.path(), "./dependencies", &repo_url, &[]);

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&project_dir)
        .arg("update")
        .assert()
        .success()
        .stdout(predicates::str::contains("Updated").or(predicates::str::contains("✅")));

    // Verify that the clone advanced: the new file must exist
    assert!(
        clone_dest.join("change.txt").exists(),
        "clone must have been fast-forwarded to include change.txt"
    );
}

/// `Nidus update` reports "up to date" when there are no new commits.
#[test]
fn test_update_reports_already_up_to_date() {
    let project_dir  = TempDir::new().unwrap();
    let upstream_dir = TempDir::new().unwrap();

    let repo_url = create_local_repo(upstream_dir.path());
    let repo_name = upstream_dir.path().file_name().unwrap().to_str().unwrap();
    let clone_dest = project_dir.path().join("dependencies").join(repo_name);
    std::fs::create_dir_all(&clone_dest).unwrap();
    clone_repo_locally(&repo_url, &clone_dest);

    // No new commit — must report already up to date
    write_nidus_json(project_dir.path(), "./dependencies", &repo_url, &[]);

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&project_dir)
        .arg("update")
        .assert()
        .success()
        .stdout(predicates::str::contains("up to date").or(predicates::str::contains("🔁")));
}

/// `Nidus update` exibe o summary com contagens ao final.
#[test]
fn test_update_shows_summary() {
    let project_dir  = TempDir::new().unwrap();
    let upstream_dir = TempDir::new().unwrap();

    let repo_url = create_local_repo(upstream_dir.path());
    let repo_name = upstream_dir.path().file_name().unwrap().to_str().unwrap();
    let clone_dest = project_dir.path().join("dependencies").join(repo_name);
    std::fs::create_dir_all(&clone_dest).unwrap();
    clone_repo_locally(&repo_url, &clone_dest);

    write_nidus_json(project_dir.path(), "./dependencies", &repo_url, &[]);

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&project_dir)
        .arg("update")
        .assert()
        .success()
        .stdout(predicates::str::contains("Update summary"));
}

/// `Nidus update` fails gracefully when the dependency folder does not exist.
#[test]
fn test_update_fails_gracefully_for_missing_clone() {
    let project_dir  = TempDir::new().unwrap();
    let upstream_dir = TempDir::new().unwrap();

    let repo_url = create_local_repo(upstream_dir.path());

    // nidus.json points to the upstream but the clone does NOT exist in ./dependencies/
    write_nidus_json(project_dir.path(), "./dependencies", &repo_url, &[]);

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&project_dir)
        .arg("update")
        .assert()
        .success() // the command does not process::exit for individual failures
        .stdout(predicates::str::contains("Failed").or(predicates::str::contains("❌")));
}

// ─── Tests: `remove module` ─────────────────────────────────────────────────

/// `remove module User --yes` removes the module directory.
#[test]
fn test_remove_module_deletes_directory() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    gen(&project_dir, "module", "User").success();

    let module_dir = project_dir.join("src").join("modules").join("user");
    assert!(module_dir.exists(), "module dir must exist before remove");

    Command::cargo_bin("Nidus").unwrap()
        .current_dir(&project_dir)
        .args(["remove", "module", "User", "--yes"])
        .assert()
        .success();

    assert!(!module_dir.exists(), "module dir must be gone after remove");
}

/// After `remove module`, UserModule and UserHandler must be absent from the .dpr.
#[test]
fn test_remove_module_cleans_dpr() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    gen(&project_dir, "module", "User").success();

    let dpr_before = read_dpr(&project_dir, "MyProject");
    assert!(dpr_before.contains("UserModule"), "UserModule must be in .dpr after gen");

    Command::cargo_bin("Nidus").unwrap()
        .current_dir(&project_dir)
        .args(["remove", "module", "User", "--yes"])
        .assert()
        .success();

    let dpr_after = read_dpr(&project_dir, "MyProject");
    assert!(!dpr_after.contains("UserModule"), "UserModule must be removed from .dpr");
    assert!(!dpr_after.contains("UserHandler"), "UserHandler must be removed from .dpr");
}

/// `remove module` with alias `rm` works the same way.
#[test]
fn test_remove_alias_rm_works() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    gen(&project_dir, "module", "User").success();

    Command::cargo_bin("Nidus").unwrap()
        .current_dir(&project_dir)
        .args(["rm", "module", "User", "--yes"])
        .assert()
        .success();

    let module_dir = project_dir.join("src").join("modules").join("user");
    assert!(!module_dir.exists(), "module dir must be gone after rm");
}

/// `remove module` fails with a non-zero exit code when the module does not exist.
#[test]
fn test_remove_module_fails_when_not_exists() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    Command::cargo_bin("Nidus").unwrap()
        .current_dir(&project_dir)
        .args(["remove", "module", "NonExistent", "--yes"])
        .assert()
        .failure();
}

/// After removing User, the .dpr `uses` block must end with `;` not `,`.
#[test]
fn test_remove_last_module_fixes_trailing_comma_in_dpr() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    gen(&project_dir, "module", "User").success();

    Command::cargo_bin("Nidus").unwrap()
        .current_dir(&project_dir)
        .args(["remove", "module", "User", "--yes"])
        .assert()
        .success();

    let dpr = read_dpr(&project_dir, "MyProject");
    // The uses block must be terminated by `;`, never by `,`
    let uses_block_end = dpr
        .find("\nbegin")
        .map(|pos| dpr[..pos].trim_end())
        .unwrap_or("");
    assert!(
        !uses_block_end.ends_with(','),
        "uses block must end with ';' not ',', got: ...{}",
        &uses_block_end[uses_block_end.len().saturating_sub(20)..]
    );
}

// ─── Tests: `gen` name validation ────────────────────────────────────────────

/// `gen module` com nome inválido (começa com número) deve falhar.
#[test]
fn test_gen_rejects_invalid_module_name_starts_with_digit() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&project_dir)
        .args(["gen", "module", "123Invalid"])
        .assert()
        .failure();
}

/// `gen module` com palavra reservada Delphi deve falhar.
#[test]
fn test_gen_rejects_delphi_reserved_word() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&project_dir)
        .args(["gen", "module", "begin"])
        .assert()
        .failure();
}

/// `gen module` com nome válido deve ter sucesso.
#[test]
fn test_gen_accepts_valid_module_name() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&project_dir)
        .args(["gen", "module", "ValidName"])
        .assert()
        .success();
}

// ─── Tests: `install --remove` ───────────────────────────────────────────────

/// `install --remove` com URL inválida deve falhar.
#[test]
fn test_install_remove_rejects_invalid_url() {
    let project_dir = TempDir::new().unwrap();
    write_nidus_json(
        project_dir.path(),
        "./dependencies",
        "https://github.com/ModernDelphiWorks/Nidus.git",
        &[],
    );

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&project_dir)
        .args(["install", "--remove", "not-a-url"])
        .assert()
        .failure();
}

/// `install --remove` sem nidus.json deve falhar com exit code 1.
#[test]
fn test_install_remove_without_nidus_json_fails() {
    let project_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&project_dir)
        .args(["install", "--remove", "https://github.com/fake/repo.git"])
        .assert()
        .failure();
}

/// `install --remove` com URL não presente no nidus.json deve falhar com mensagem.
#[test]
fn test_install_remove_nonexistent_url_fails() {
    let project_dir = TempDir::new().unwrap();
    write_nidus_json(
        project_dir.path(),
        "./dependencies",
        "https://github.com/ModernDelphiWorks/Nidus.git",
        &[],
    );

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&project_dir)
        .args(["install", "--remove", "https://github.com/fake/nonexistent.git"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("not found in nidus.json"));
}

/// `install --remove` remove a dependência do nidus.json.
#[test]
fn test_install_remove_removes_dependency_from_nidus_json() {
    let project_dir = TempDir::new().unwrap();
    let dep_url = "https://github.com/fake-user/to-remove.git";

    write_nidus_json(
        project_dir.path(),
        "./dependencies",
        "https://github.com/ModernDelphiWorks/Nidus.git",
        &[(dep_url, "")],
    );

    // Verificar que está presente antes
    let before = std::fs::read_to_string(project_dir.path().join("nidus.json")).unwrap();
    assert!(before.contains(dep_url));

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&project_dir)
        .args(["install", "--remove", dep_url])
        .assert()
        .success()
        .stdout(predicates::str::contains("Removed"));

    // Verificar que foi removida
    let after = std::fs::read_to_string(project_dir.path().join("nidus.json")).unwrap();
    assert!(
        !after.contains(dep_url),
        "URL deve ter sido removida do nidus.json, got:\n{}",
        after
    );
}

// ─── Tests: `template create --from` ─────────────────────────────────────────

/// `template create --from` cria template.json com arquivos .pas substituídos.
#[test]
fn test_template_create_from_scans_pas_files() {
    let temp_dir = TempDir::new().unwrap();

    // Cria diretório de módulo simulado
    let module_dir = temp_dir.path().join("user");
    std::fs::create_dir_all(&module_dir).unwrap();
    std::fs::write(
        module_dir.join("UserModule.pas"),
        "unit UserModule;\ntype TUserModule = class end;",
    ).unwrap();
    std::fs::write(
        module_dir.join("UserService.pas"),
        "unit UserService;\ntype TUserService = class end;",
    ).unwrap();

    // Usar uma home temporária para não poluir ~/.Nidus
    let home_dir = temp_dir.path().join("home");
    std::fs::create_dir_all(&home_dir).unwrap();

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("HOME", &home_dir)
        .args(["template", "create", "user-template", "--from", "user"])
        .assert()
        .success();

    // template.json deve existir
    let template_json_path = home_dir
        .join(".Nidus")
        .join("templates")
        .join("user-template")
        .join("template.json");
    assert!(template_json_path.exists(), "template.json deve ter sido criado");

    let json = std::fs::read_to_string(&template_json_path).unwrap();
    // Placeholder {{mod}} deve aparecer no campo `path` e no `content`
    assert!(json.contains("{{mod}}"), "placeholder {{{{mod}}}} deve estar no template.json");
    // O campo `name` preserva o nome original para exibição — verificar que `path` foi substituído
    // O JSON contém campos: "name": "UserModule.pas", "path": "{{mod}}Module.pas"
    // Verificar que o `path` foi parametrizado
    assert!(
        json.contains("\"path\": \"{{mod}}"),
        "campo path deve ter placeholder, got:\n{}",
        &json[..json.len().min(500)]
    );
}

// ─── Tests: `doctor` command ─────────────────────────────────────────────────

/// `doctor` exibe todas as seções A–E e termina com exit 0.
#[test]
fn test_doctor_shows_all_sections() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&project_dir)
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicates::str::contains("A. Configuration"))
        .stdout(predicates::str::contains("B. Project Structure"))
        .stdout(predicates::str::contains("C. Dependencies"))
        .stdout(predicates::str::contains("D. Module Consistency"))
        .stdout(predicates::str::contains("E. Environment"));
}

/// `doctor` em projeto com nidus.json e .dpr exibe ✅ para ambos.
#[test]
fn test_doctor_with_valid_project_detects_dpr_and_config() {
    let project_dir = TempDir::new().unwrap();

    // nidus.json com URL local válida (não precisa clonar)
    let upstream = TempDir::new().unwrap();
    let repo_url = create_local_repo(upstream.path());
    write_nidus_json(project_dir.path(), "./src", &repo_url, &[]);

    // Criar estrutura mínima de projeto
    std::fs::create_dir_all(project_dir.path().join("src").join("modules")).unwrap();
    std::fs::write(
        project_dir.path().join("src").join("AppModule.pas"),
        "unit AppModule;\nuses\n  BaseModule;\nend.",
    ).unwrap();
    std::fs::write(
        project_dir.path().join("MyProject.dpr"),
        "program MyProject;\nbegin\nend.",
    ).unwrap();

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&project_dir)
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicates::str::contains("nidus.json"))
        .stdout(predicates::str::contains(".dpr"));
}

/// `doctor` sem nidus.json reporta erro na seção A.
#[test]
fn test_doctor_without_nidus_json_reports_error() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&temp_dir)
        .arg("doctor")
        .assert()
        .success() // doctor não faz exit 1 — reporta e sai com sucesso
        .stdout(predicates::str::contains("nidus.json"));
}

/// `doctor` com .dpr mostra verificação D1.
#[test]
fn test_doctor_checks_dpr_unit_files() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    // Gerar um módulo para ter .dpr com unidades registradas
    Command::cargo_bin("Nidus").unwrap()
        .current_dir(&project_dir)
        .args(["gen", "module", "User"])
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("Nidus").unwrap();
    cmd.current_dir(&project_dir)
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicates::str::contains("unit"));
}

// ─── Tests: `init` command ────────────────────────────────────────────────────

/// `init` em diretório sem nidus.json cria o arquivo.
#[test]
fn test_init_creates_nidus_json() {
    let temp_dir = TempDir::new().unwrap();

    assert!(!temp_dir.path().join("nidus.json").exists());

    Command::cargo_bin("Nidus").unwrap()
        .current_dir(&temp_dir)
        .arg("init")
        .assert()
        .success();

    assert!(
        temp_dir.path().join("nidus.json").exists(),
        "nidus.json deve ser criado pelo comando init"
    );
}

/// `init` quando nidus.json já existe deve falhar sem --force.
#[test]
fn test_init_fails_if_nidus_json_exists() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("nidus.json"), r#"{"name":"existing"}"#).unwrap();

    Command::cargo_bin("Nidus").unwrap()
        .current_dir(&temp_dir)
        .arg("init")
        .assert()
        .failure()
        .stderr(predicates::str::contains("already exists"));
}

/// `init --force` sobrescreve nidus.json existente.
#[test]
fn test_init_force_overwrites() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("nidus.json"), r#"{"name":"old"}"#).unwrap();

    Command::cargo_bin("Nidus").unwrap()
        .current_dir(&temp_dir)
        .args(["init", "--force"])
        .assert()
        .success();

    let content = std::fs::read_to_string(temp_dir.path().join("nidus.json")).unwrap();
    assert!(
        !content.contains("\"name\":\"old\""),
        "nidus.json deve ter sido sobrescrito pelo --force"
    );
}

/// `init --mainsrc` grava o valor personalizado no nidus.json criado.
#[test]
fn test_init_custom_mainsrc() {
    let temp_dir = TempDir::new().unwrap();

    Command::cargo_bin("Nidus").unwrap()
        .current_dir(&temp_dir)
        .args(["init", "--mainsrc", "lib/"])
        .assert()
        .success();

    let content = std::fs::read_to_string(temp_dir.path().join("nidus.json")).unwrap();
    assert!(
        content.contains("lib/"),
        "nidus.json deve conter o mainsrc 'lib/' informado, got:\n{}",
        content
    );
}

// ─── Tests: `clean` command ───────────────────────────────────────────────────

/// dry-run lista artefatos mas não remove.
#[test]
fn test_clean_dry_run_lists_artifacts() {
    let temp_dir = TempDir::new().unwrap();
    let dcu_path = temp_dir.path().join("MyUnit.dcu");
    std::fs::write(&dcu_path, b"fake dcu content").unwrap();

    Command::cargo_bin("Nidus").unwrap()
        .current_dir(&temp_dir)
        .arg("clean")
        .assert()
        .success()
        .stdout(predicates::str::contains("MyUnit.dcu"));

    assert!(
        dcu_path.exists(),
        "arquivo .dcu não deve ser removido no modo dry-run"
    );
}

/// `clean --execute --yes` remove o arquivo .dcu.
#[test]
fn test_clean_execute_removes_dcu() {
    let temp_dir = TempDir::new().unwrap();
    let dcu_path = temp_dir.path().join("SomeUnit.dcu");
    std::fs::write(&dcu_path, b"fake dcu content").unwrap();

    Command::cargo_bin("Nidus").unwrap()
        .current_dir(&temp_dir)
        .args(["clean", "--execute", "--yes"])
        .assert()
        .success();

    assert!(
        !dcu_path.exists(),
        "arquivo .dcu deve ser removido com --execute --yes"
    );
}

/// `clean` em pasta sem artefatos exibe "Nothing to clean".
#[test]
fn test_clean_no_artifacts() {
    let temp_dir = TempDir::new().unwrap();

    Command::cargo_bin("Nidus").unwrap()
        .current_dir(&temp_dir)
        .arg("clean")
        .assert()
        .success()
        .stdout(predicates::str::contains("Nothing to clean"));
}

// ─── Tests: `deps` command ────────────────────────────────────────────────────

/// `deps` sem nidus.json deve falhar com mensagem.
#[test]
fn test_deps_requires_nidus_json() {
    let temp_dir = TempDir::new().unwrap();

    Command::cargo_bin("Nidus").unwrap()
        .current_dir(&temp_dir)
        .arg("deps")
        .assert()
        .failure()
        .stderr(predicates::str::contains("nidus.json"));
}

/// `deps` com nidus.json contendo dependências exibe o nome do repositório extraído da URL.
#[test]
fn test_deps_shows_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    write_nidus_json(
        temp_dir.path(),
        "./dependencies",
        "https://github.com/HashLoad/Horse.git",
        &[("https://github.com/ModernDelphiWorks/InjectContainer.git", "")],
    );

    Command::cargo_bin("Nidus").unwrap()
        .current_dir(&temp_dir)
        .arg("deps")
        .assert()
        .success()
        .stdout(predicates::str::contains("Horse"));
}

/// `deps --json` produz JSON válido parseável por serde_json.
#[test]
fn test_deps_json_flag() {
    let temp_dir = TempDir::new().unwrap();
    write_nidus_json(
        temp_dir.path(),
        "./dependencies",
        "https://github.com/HashLoad/Horse.git",
        &[],
    );

    let output = Command::cargo_bin("Nidus").unwrap()
        .current_dir(&temp_dir)
        .args(["deps", "--json"])
        .output()
        .unwrap();

    assert!(output.status.success(), "deps --json deve terminar com sucesso");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
    assert!(
        parsed.is_ok(),
        "output de deps --json deve ser JSON válido, got:\n{}",
        stdout
    );
}

// ─── Tests: `outdated` command ────────────────────────────────────────────────

/// `outdated` sem nidus.json exibe mensagem de erro no stderr (processo termina com sucesso — usa return).
#[test]
fn test_outdated_requires_nidus_json() {
    let temp_dir = TempDir::new().unwrap();

    Command::cargo_bin("Nidus").unwrap()
        .current_dir(&temp_dir)
        .arg("outdated")
        .assert()
        .success()
        .stderr(predicates::str::contains("nidus.json").or(predicates::str::contains("not loaded")));
}

/// `outdated` com nidus.json válido mas sem dependências clonadas não crasha.
#[test]
fn test_outdated_runs_with_valid_config() {
    let temp_dir = TempDir::new().unwrap();
    write_nidus_json(
        temp_dir.path(),
        "./dependencies",
        "https://github.com/HashLoad/Horse.git",
        &[],
    );

    Command::cargo_bin("Nidus").unwrap()
        .current_dir(&temp_dir)
        .arg("outdated")
        .assert()
        .success()
        .stdout(
            predicates::str::contains("could not check")
                .or(predicates::str::contains("not cloned"))
                .or(predicates::str::contains("Checking"))
                .or(predicates::str::contains("Summary")),
        );
}

// ─── Tests: `doctor --fix` ────────────────────────────────────────────────────

/// `doctor --fix` em projeto sem nidus.json não crasha e exibe resultado.
#[test]
fn test_doctor_fix_flag_accepted() {
    let temp_dir = TempDir::new().unwrap();

    Command::cargo_bin("Nidus").unwrap()
        .current_dir(&temp_dir)
        .args(["doctor", "--fix"])
        .assert()
        .success()
        .stdout(
            predicates::str::contains("Auto-fixing")
                .or(predicates::str::contains("nothing fixable"))
                .or(predicates::str::contains("nidus.json")),
        );
}

// ─── Tests: `nidus.lock` e `install --frozen` ─────────────────────────────────

/// Sem rodar install, nidus.lock não deve existir.
#[test]
fn test_lock_file_not_created_without_install() {
    let temp_dir = TempDir::new().unwrap();
    write_nidus_json(
        temp_dir.path(),
        "./dependencies",
        "https://github.com/HashLoad/Horse.git",
        &[],
    );

    assert!(
        !temp_dir.path().join("nidus.lock").exists(),
        "nidus.lock não deve existir antes de rodar install"
    );
}

/// `install --frozen` sem nidus.lock falha com mensagem "nidus.lock not found".
#[test]
fn test_install_frozen_fails_without_lock() {
    let temp_dir = TempDir::new().unwrap();
    write_nidus_json(
        temp_dir.path(),
        "./dependencies",
        "https://github.com/HashLoad/Horse.git",
        &[],
    );

    Command::cargo_bin("Nidus").unwrap()
        .current_dir(&temp_dir)
        .args(["install", "--frozen"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("nidus.lock not found"));
}

// ─── Tests: `gen --interactive` (TTY guard) ───────────────────────────────────

/// `gen module --interactive` via Command (sem TTY) exibe mensagem sobre TTY.
/// O processo termina com sucesso (return, não exit 1) mas emite a mensagem no stderr.
#[test]
fn test_gen_interactive_fails_without_tty() {
    let temp_dir = TempDir::new().unwrap();
    setup_project(&temp_dir, "MyProject");
    let project_dir = temp_dir.path().join("MyProject");

    Command::cargo_bin("Nidus").unwrap()
        .current_dir(&project_dir)
        .args(["gen", "module", "Foo", "--interactive"])
        .assert()
        .stderr(predicates::str::contains("TTY").or(predicates::str::contains("interactive")));
}
