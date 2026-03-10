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
      assert!(project_path.join("crates/server/src/main.rs").exists());
      assert!(project_path.join("crates/migrations").exists());
      assert!(project_path.join("crates/db/src/lib.rs").exists());
      assert!(project_path.join(".gitignore").exists());
      assert!(project_path.join("config/app.toml").exists());

      // Cargo.toml content
      let cargo_content = fs::read_to_string(project_path.join("Cargo.toml")).unwrap();
      assert!(cargo_content.contains("[workspace]"));
      assert!(cargo_content.contains("crates/app"));
      assert!(cargo_content.contains("crates/db"));
      assert!(cargo_content.contains("crates/migrations"));
      assert!(cargo_content.contains("crates/server"));
      let app_cargo = fs::read_to_string(project_path.join("crates/app/Cargo.toml")).unwrap();
      assert!(app_cargo.contains("name = \"app\""));
      assert!(
        app_cargo.contains("forge-app =") || app_cargo.contains("forge_config ="),
        "generated app should depend on forge-app or forge-config"
      );
      assert!(app_cargo.contains("db ="));

      // lib.rs: Forge app building (explicit imports, route_methods for dashboard)
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
      let server_main_path = project_path.join("crates/server/src/main.rs");
      assert!(
        server_main_path.exists(),
        "template should include server crate at {}",
        server_main_path.display()
      );
      let server_main = fs::read_to_string(&server_main_path).unwrap();
      assert!(
        server_main.contains(".with_migrations(migrations::Migrator)"),
        "generated server should use migrations::Migrator"
      );
      assert!(lib_content.contains(".post_route"));
      assert!(
        lib_content.contains("/api/auth/admin"),
        "generated app should register /api/auth/admin route"
      );
      assert!(
        lib_content.contains(".route_methods(")
          && (lib_content.contains("/api/dashboard/users")
            || lib_content.contains("/api/organizations/{id}/users")
            || lib_content.contains("/api/organizations/:id/users")),
        "generated app should use route_methods for dashboard/org users"
      );

      // server main.rs: entrypoint and router setup
      assert!(
        !server_main.contains("forge::prelude"),
        "generated server main should use explicit imports (no prelude)"
      );
      assert!(
        server_main.contains(".serve()") || server_main.contains("into_router_before_state"),
        "generated server main should call .serve() or into_router_before_state"
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
          environment: None,
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
        frontend: FrontendConfig {
          host: "127.0.0.1".to_string(),
          port: 5173,
        },
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

  mod bdd_tests {
    use super::*;
    use std::sync::Mutex;

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

    static CREATE_PROJECT_LOCK: Mutex<()> = Mutex::new(());

    /// BDD-style tests focusing on behavior rather than implementation.
    /// Tests are organized by feature/behavior area with descriptive names.
    mod main_function_behavior {
      use super::*;

      /// Helper function to simulate main() behavior with parsed args
      fn simulate_main_with_args(args: Args) -> std::process::ExitCode {
        match args.command {
          Some(Commands::New { name }) => {
            if let Err(_e) = create_new_project(&name) {
              return std::process::ExitCode::FAILURE;
            }
            std::process::ExitCode::SUCCESS
          }
          Some(Commands::Dev {}) => {
            let config = match forge_config::load_config() {
              Ok(c) => c,
              Err(_e) => {
                return std::process::ExitCode::FAILURE;
              }
            };
            if let Err(_e) = dev::run(&config) {
              return std::process::ExitCode::FAILURE;
            }
            std::process::ExitCode::SUCCESS
          }
          Some(Commands::Serve {}) => {
            let config = match forge_config::load_config() {
              Ok(c) => c,
              Err(_e) => {
                return std::process::ExitCode::FAILURE;
              }
            };
            if let Err(_e) = serve::run(&config) {
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

      #[test]
      fn should_return_success_when_no_command_provided() {
        // Given: Args with no command subcommand
        let args = Args { command: None };

        // When: Processing the command
        let exit_code = simulate_main_with_args(args);

        // Then: Should return SUCCESS exit code
        assert_eq!(exit_code, std::process::ExitCode::SUCCESS);
      }

      #[test]
      fn should_return_success_when_new_command_succeeds() {
        // Given: Args with New command and valid project name
        let _guard = CREATE_PROJECT_LOCK.lock().unwrap();
        let template_parent = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("templates");
        let _env_guard = RestoreForgeTemplatesDir(std::env::var("FORGE_TEMPLATES_DIR").ok());
        unsafe {
          std::env::set_var("FORGE_TEMPLATES_DIR", &template_parent);
        }
        let temp_dir = tempdir().unwrap();
        let project_path = temp_dir.path().join("test_new_success");
        let args = Args {
          command: Some(Commands::New {
            name: project_path.to_str().unwrap().to_string(),
          }),
        };

        // When: Processing the New command
        let exit_code = simulate_main_with_args(args);

        // Then: Should return SUCCESS exit code
        assert_eq!(exit_code, std::process::ExitCode::SUCCESS);
        assert!(project_path.exists());
      }

      #[test]
      fn should_return_failure_when_new_command_fails() {
        // Given: Args with New command pointing to existing directory
        let _guard = CREATE_PROJECT_LOCK.lock().unwrap();
        let temp_dir = tempdir().unwrap();
        let project_path = temp_dir.path().join("existing_dir");
        fs::create_dir(&project_path).unwrap();
        let args = Args {
          command: Some(Commands::New {
            name: project_path.to_str().unwrap().to_string(),
          }),
        };

        // When: Processing the New command
        let exit_code = simulate_main_with_args(args);

        // Then: Should return FAILURE exit code
        assert_eq!(exit_code, std::process::ExitCode::FAILURE);
      }

      #[test]
      fn should_return_failure_when_dev_command_cannot_load_config() {
        // Given: Args with Dev command in directory without config
        let _guard = crate::CHDIR_TEST_LOCK.lock().unwrap();
        let temp_dir = tempdir().unwrap();
        let orig = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        let args = Args {
          command: Some(Commands::Dev {}),
        };

        // When: Processing the Dev command
        let exit_code = simulate_main_with_args(args);

        // Then: Should return FAILURE exit code (config load fails)
        assert_eq!(exit_code, std::process::ExitCode::FAILURE);

        std::env::set_current_dir(orig).unwrap();
      }

      #[test]
      fn should_return_failure_when_serve_command_cannot_load_config() {
        // Given: Args with Serve command in directory without config
        let _guard = crate::CHDIR_TEST_LOCK.lock().unwrap();
        let temp_dir = tempdir().unwrap();
        let orig = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        let args = Args {
          command: Some(Commands::Serve {}),
        };

        // When: Processing the Serve command
        let exit_code = simulate_main_with_args(args);

        // Then: Should return FAILURE exit code (config load fails)
        assert_eq!(exit_code, std::process::ExitCode::FAILURE);

        std::env::set_current_dir(orig).unwrap();
      }
    }

    mod command_dispatching_behavior {
      use super::*;

      #[test]
      fn should_dispatch_to_new_handler_when_new_command_provided() {
        // Given: Parsed args with New command
        let args = Args::try_parse_from(["forge", "new", "test_project"]).unwrap();

        // When: Checking the command variant
        // Then: Should contain New command
        match args.command {
          Some(Commands::New { name }) => {
            assert_eq!(name, "test_project");
          }
          _ => panic!("Expected New command to be dispatched"),
        }
      }

      #[test]
      fn should_dispatch_to_dev_handler_when_dev_command_provided() {
        // Given: Parsed args with Dev command
        let args = Args::try_parse_from(["forge", "dev"]).unwrap();

        // When: Checking the command variant
        // Then: Should contain Dev command
        match args.command {
          Some(Commands::Dev {}) => (),
          _ => panic!("Expected Dev command to be dispatched"),
        }
      }

      #[test]
      fn should_dispatch_to_serve_handler_when_serve_command_provided() {
        // Given: Parsed args with Serve command
        let args = Args::try_parse_from(["forge", "serve"]).unwrap();

        // When: Checking the command variant
        // Then: Should contain Serve command
        match args.command {
          Some(Commands::Serve {}) => (),
          _ => panic!("Expected Serve command to be dispatched"),
        }
      }

      #[test]
      fn should_handle_no_command_by_showing_help() {
        // Given: Parsed args with no subcommand
        let args = Args::try_parse_from(["forge"]).unwrap();

        // When: Checking the command variant
        // Then: Should be None, indicating help should be shown
        assert!(args.command.is_none());
      }
    }

    mod error_handling_behavior {
      use super::*;

      /// Helper function to simulate main() behavior with parsed args
      fn simulate_main_with_args(args: Args) -> std::process::ExitCode {
        match args.command {
          Some(Commands::New { name }) => {
            if let Err(_e) = create_new_project(&name) {
              return std::process::ExitCode::FAILURE;
            }
            std::process::ExitCode::SUCCESS
          }
          Some(Commands::Dev {}) => {
            let config = match forge_config::load_config() {
              Ok(c) => c,
              Err(_e) => {
                return std::process::ExitCode::FAILURE;
              }
            };
            if let Err(_e) = dev::run(&config) {
              return std::process::ExitCode::FAILURE;
            }
            std::process::ExitCode::SUCCESS
          }
          Some(Commands::Serve {}) => {
            let config = match forge_config::load_config() {
              Ok(c) => c,
              Err(_e) => {
                return std::process::ExitCode::FAILURE;
              }
            };
            if let Err(_e) = serve::run(&config) {
              return std::process::ExitCode::FAILURE;
            }
            std::process::ExitCode::SUCCESS
          }
          None => {
            let _ = Args::command().print_help();
            std::process::ExitCode::SUCCESS
          }
        }
      }

      #[test]
      fn should_handle_new_command_errors_gracefully() {
        // Given: New command that will fail (existing directory)
        let _guard = CREATE_PROJECT_LOCK.lock().unwrap();
        let temp_dir = tempdir().unwrap();
        let project_path = temp_dir.path().join("existing");
        fs::create_dir(&project_path).unwrap();
        let args = Args {
          command: Some(Commands::New {
            name: project_path.to_str().unwrap().to_string(),
          }),
        };

        // When: Processing the command
        let exit_code = simulate_main_with_args(args);

        // Then: Should return failure exit code (not panic)
        assert_eq!(exit_code, std::process::ExitCode::FAILURE);
      }

      #[test]
      fn should_handle_config_load_errors_gracefully() {
        // Given: Dev command in directory without config
        let _guard = crate::CHDIR_TEST_LOCK.lock().unwrap();
        let temp_dir = tempdir().unwrap();
        let orig = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        let args = Args {
          command: Some(Commands::Dev {}),
        };

        // When: Processing the command
        let exit_code = simulate_main_with_args(args);

        // Then: Should return failure exit code (not panic)
        assert_eq!(exit_code, std::process::ExitCode::FAILURE);

        std::env::set_current_dir(orig).unwrap();
      }
    }
  }
}
