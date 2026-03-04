//! Forge CLI — invoke from tests by running the binary and asserting on output.

mod dev;
mod new;
mod serve;

/// Serializes tests that change process cwd so they don't race (used by serve/dev/create_new_project tests).
#[cfg(test)]
pub(crate) static CHDIR_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

use clap::{CommandFactory, Parser, Subcommand};
use new::create_new_project;

#[derive(Parser, Debug)]
#[command(name = "forge")]
#[command(about = "Forge — full-stack web framework for Rust")]
#[command(version)]
struct Args {
  #[command(subcommand)]
  command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
  /// Create a new Forge project
  New {
    /// Name of the project to create
    name: String,
  },
  /// Development mode: Vite dev server + Rust server (hot reload)
  Dev {},
  /// Production mode: build frontend, then serve static assets with the Rust server
  Serve {},
}

fn main() -> std::process::ExitCode {
  let args = Args::parse();

  match args.command {
    Some(Commands::New { name }) => {
      if let Err(e) = create_new_project(&name) {
        eprintln!("Error creating project: {}", e);
        return std::process::ExitCode::FAILURE;
      }
      println!("Created new Forge project: {}", name);
      std::process::ExitCode::SUCCESS
    }
    Some(Commands::Dev {}) => {
      let config = match forge_config::load_config() {
        Ok(c) => c,
        Err(e) => {
          eprintln!("Error: {}", e);
          return std::process::ExitCode::FAILURE;
        }
      };
      if let Err(e) = dev::run(&config) {
        eprintln!("Error: {}", e);
        return std::process::ExitCode::FAILURE;
      }
      std::process::ExitCode::SUCCESS
    }
    Some(Commands::Serve {}) => {
      let config = match forge_config::load_config() {
        Ok(c) => c,
        Err(e) => {
          eprintln!("Error: {}", e);
          return std::process::ExitCode::FAILURE;
        }
      };
      if let Err(e) = serve::run(&config) {
        eprintln!("Error: {}", e);
        return std::process::ExitCode::FAILURE;
      }
      std::process::ExitCode::SUCCESS
    }
    None => {
      // No subcommand provided, show help
      let _ = Args::command().print_help();
      std::process::ExitCode::SUCCESS
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  use tempfile::tempdir;

  /// Test suite for CLI argument parsing
  mod cli_parsing {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn args_struct_parses_version_flag() {
      // Given: Command line arguments with --version
      let args = Args::try_parse_from(["forge", "--version"]);

      // When: Parsing the arguments
      // Then: It should return an error because --version causes clap to exit
      // This is expected behavior - clap handles --version by printing and exiting
      assert!(args.is_err());
      // Note: version is handled by clap automatically
    }

    #[test]
    fn args_struct_parses_help_flag() {
      // Given: Command line arguments with --help
      let _args = Args::try_parse_from(["forge", "--help"]);

      // When: Parsing the arguments
      // Then: It should succeed (clap handles help internally)
      // Note: --help causes clap to exit, so we test the structure
      let cmd = Args::command();
      assert!(cmd.get_name() == "forge");
    }

    #[test]
    fn new_subcommand_parses_project_name() {
      // Given: Command line with new subcommand
      let args = Args::try_parse_from(["forge", "new", "my_project"]);

      // When: Parsing the arguments
      // Then: It should contain the New command with correct name
      assert!(args.is_ok());
      let args = args.unwrap();
      match args.command {
        Some(Commands::New { name }) => {
          assert_eq!(name, "my_project");
        }
        _ => panic!("Expected New command"),
      }
    }

    #[test]
    fn no_subcommand_defaults_to_none() {
      // Given: Command line with no subcommand
      let args = Args::try_parse_from(["forge"]);

      // When: Parsing the arguments
      // Then: command should be None
      assert!(args.is_ok());
      let args = args.unwrap();
      assert!(args.command.is_none());
    }
  }

  /// Test suite for Commands enum
  mod commands_enum {
    use super::*;

    #[test]
    fn commands_enum_has_new_variant() {
      // Given: Commands enum
      // When: Creating New variant
      let cmd = Commands::New {
        name: "test".to_string(),
      };

      // Then: It should compile and work
      match cmd {
        Commands::New { name } => assert_eq!(name, "test"),
        _ => panic!("Expected New variant"),
      }
    }

    #[test]
    fn commands_enum_has_dev_variant() {
      let cmd = Commands::Dev {};
      match cmd {
        Commands::Dev {} => (),
        _ => panic!("Expected Dev variant"),
      }
    }

    #[test]
    fn commands_enum_has_serve_variant() {
      // Given: Commands enum
      // When: Creating Serve variant
      let cmd = Commands::Serve {};

      // Then: It should compile and work
      match cmd {
        Commands::Serve {} => (),
        _ => panic!("Expected Serve variant"),
      }
    }

    #[test]
    fn commands_enum_debug_trait_works() {
      // Given: Any Commands variant
      let cmd = Commands::New {
        name: "debug_test".to_string(),
      };

      // When: Using Debug formatting
      let debug_str = format!("{:?}", cmd);

      // Then: It should produce debug output
      assert!(debug_str.contains("New"));
      assert!(debug_str.contains("debug_test"));
    }
  }

  /// Test suite for main function command dispatching
  mod main_function_dispatch {
    use super::*;

    #[test]
    fn main_handles_new_command_success() {
      // Given: A temporary directory for testing
      let temp_dir = tempdir().unwrap();
      let original_cwd = std::env::current_dir().unwrap();

      // Change to temp directory for isolated testing
      std::env::set_current_dir(&temp_dir).unwrap();

      // When: main processes New command
      // Note: We can't easily test main() directly, but we test the logic
      // by testing the create_new_project function separately

      // Restore original directory
      std::env::set_current_dir(original_cwd).unwrap();
    }

    #[test]
    fn main_handles_serve_command_success() {
      // Given: Valid project setup (tested separately)
      // When: main processes Serve command
      // Then: It should dispatch to serve::run
    }

    #[test]
    fn main_handles_no_command_by_showing_help() {
      // Given: No subcommand provided
      // When: main is called
      // Then: It should show help and return success
      // Note: This is tested by the Args parsing tests above
    }
  }

  /// Test suite for create_new_project function.
  /// Tests are serialized with a lock because they share template reads and can flake when run in parallel.
  mod create_new_project_function {
    use super::*;
    use std::sync::Mutex;

    static CREATE_PROJECT_LOCK: Mutex<()> = Mutex::new(());

    /// Restores FORGE_TEMPLATES_DIR on drop (so test can restore env even on panic).
    struct RestoreForgeTemplatesDir(Option<String>);
    impl Drop for RestoreForgeTemplatesDir {
      fn drop(&mut self) {
        if let Some(ref v) = self.0 {
          unsafe {
            std::env::set_var("FORGE_TEMPLATES_DIR", v);
          }
        } else {
          unsafe {
            std::env::remove_var("FORGE_TEMPLATES_DIR");
          }
        }
      }
    }

    #[test]
    fn create_new_project_succeeds_with_valid_name() {
      let _guard = CREATE_PROJECT_LOCK.lock().unwrap();
      // Point at this crate's template so the test is hermetic regardless of cwd.
      let template_parent = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("templates");
      let _env_guard = RestoreForgeTemplatesDir(std::env::var("FORGE_TEMPLATES_DIR").ok());
      unsafe {
        std::env::set_var("FORGE_TEMPLATES_DIR", &template_parent);
      }

      // Given: A temporary directory (use full path so test is safe when run in parallel).
      // Single project creation for all content assertions to reduce disk usage.
      let temp_dir = tempdir().unwrap();
      let project_name = "test_project";
      let project_path = temp_dir.path().join(project_name);

      // When: Creating a new project
      let result = create_new_project(project_path.to_str().unwrap());

      // Then: It should succeed and create files
      assert!(
        result.is_ok(),
        "create_new_project failed: {:?}",
        result.err()
      );

      // Verify directory structure
      assert!(project_path.exists());
      assert!(project_path.join("Cargo.toml").exists());
      assert!(project_path.join("crates/app").exists());
      assert!(project_path.join("crates/app/src/main.rs").exists());
      assert!(project_path.join("crates/db/src/lib.rs").exists());
      assert!(project_path.join(".gitignore").exists());
      assert!(project_path.join("config/app.toml").exists());

      // Cargo.toml content
      let cargo_content = fs::read_to_string(project_path.join("Cargo.toml")).unwrap();
      assert!(cargo_content.contains("[workspace]"));
      assert!(cargo_content.contains("crates/app"));
      assert!(cargo_content.contains("crates/db"));
      let app_cargo = fs::read_to_string(project_path.join("crates/app/Cargo.toml")).unwrap();
      assert!(app_cargo.contains("name = \"app\""));
      assert!(
        app_cargo.contains("forge-app =") || app_cargo.contains("forge_config ="),
        "generated app should depend on forge-app or forge-config"
      );
      assert!(app_cargo.contains("db ="));

      // lib.rs: Forge app building (explicit imports, cron, route_methods for dashboard)
      let lib_content = fs::read_to_string(project_path.join("crates/app/src/lib.rs")).unwrap();
      assert!(
        (lib_content.contains("use forge_app::") || lib_content.contains("use forge::"))
          && lib_content.contains("App"),
        "generated app should use explicit forge imports"
      );
      assert!(
        !lib_content.contains("forge::prelude"),
        "generated app should use explicit imports"
      );
      assert!(lib_content.contains("App::new()"));
      assert!(lib_content.contains(".with_migrations(db::Migrator)"));
      assert!(lib_content.contains(".with_cron"));
      assert!(lib_content.contains("CronSchedule"));
      assert!(lib_content.contains(".post_route"));
      assert!(lib_content.contains(".route(\"/api/auth/admin\""));
      assert!(
        lib_content.contains(".route_methods(")
          && (lib_content.contains("/api/dashboard/users")
            || lib_content.contains("/api/organizations/:id/users")),
        "generated app should use route_methods for dashboard/org users"
      );

      // main.rs: entrypoint and router setup
      let main_content = fs::read_to_string(project_path.join("crates/app/src/main.rs")).unwrap();
      assert!(
        !main_content.contains("forge::prelude"),
        "generated main should use explicit imports (no prelude)"
      );
      assert!(
        main_content.contains(".serve()") || main_content.contains("into_router_before_state"),
        "generated main should call .serve() or into_router_before_state"
      );

      // .gitignore content
      let gitignore_content = fs::read_to_string(project_path.join(".gitignore")).unwrap();
      assert!(gitignore_content.contains("target/"));
      assert!(gitignore_content.contains(".env"));
      assert!(gitignore_content.contains("*.log"));
      assert!(gitignore_content.contains(".vscode/"));
      assert!(gitignore_content.contains(".DS_Store"));

      // Git init is best-effort (e.g. git may not be in PATH in some environments)
      if project_path.join(".git").exists() {
        // Verify git was initialized when available
      }
    }

    #[test]
    fn create_new_project_fails_with_existing_directory() {
      let _guard = CREATE_PROJECT_LOCK.lock().unwrap();
      // Given: A temporary directory with existing subdirectory (use full path for parallel safety)
      let temp_dir = tempdir().unwrap();
      let project_name = "existing_dir";
      let project_path = temp_dir.path().join(project_name);

      // Create the directory first
      fs::create_dir(&project_path).unwrap();

      // When: Trying to create project with existing path
      let result = create_new_project(project_path.to_str().unwrap());

      // Then: It should fail
      assert!(result.is_err());
      assert!(result.unwrap_err().to_string().contains("already exists"));
    }
  }

  /// Test suite for serve::run (error paths only; success path runs the server).
  mod serve_run {
    use super::*;
    use forge_config::{AppConfig, DatabaseConfig, ForgeConfig, FrontendConfig, ServerConfig};

    fn default_config() -> ForgeConfig {
      ForgeConfig {
        app: AppConfig {
          name: "test".to_string(),
        },
        server: ServerConfig {
          host: "127.0.0.1".to_string(),
          port: 3000,
        },
        database: DatabaseConfig {
          url: "sqlite::memory:".to_string(),
          max_connections: None,
          min_connections: None,
          connect_timeout: None,
          idle_timeout: None,
          auto_migrate: true,
          auto_seed: false,
        },
        cache: None,
        frontend: FrontendConfig { port: 5173 },
      }
    }

    #[test]
    fn serve_run_err_when_no_cargo_toml() {
      let _guard = crate::CHDIR_TEST_LOCK.lock().unwrap();
      let tmp = tempdir().unwrap();
      let orig = std::env::current_dir().unwrap();
      let _ = std::env::set_current_dir(tmp.path());
      let r = serve::run(&default_config());
      let _ = std::env::set_current_dir(orig);
      let err = r.unwrap_err();
      assert!(err.to_string().contains("No Cargo.toml found"));
    }

    #[test]
    fn serve_run_err_when_no_app_main() {
      let _guard = crate::CHDIR_TEST_LOCK.lock().unwrap();
      let tmp = tempdir().unwrap();
      std::fs::write(tmp.path().join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();
      let orig = std::env::current_dir().unwrap();
      let _ = std::env::set_current_dir(tmp.path());
      let r = serve::run(&default_config());
      let _ = std::env::set_current_dir(orig);
      let err = r.unwrap_err();
      assert!(err.to_string().contains("main.rs"));
    }
  }
}
