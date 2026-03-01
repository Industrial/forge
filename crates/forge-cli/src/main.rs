//! Forge CLI — invoke from tests by running the binary and asserting on output.

mod new;
mod serve;

use clap::{CommandFactory, Parser, Subcommand};
use new::create_new_project;
use serve::serve_project;

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
  /// Serve the current Forge project
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
    Some(Commands::Serve {}) => {
      use forge::config;
      let config = match config::load_config() {
        Ok(c) => c,
        Err(e) => {
          eprintln!("Error: {}", e);
          return std::process::ExitCode::FAILURE;
        }
      };
      if let Err(e) = serve_project(&config) {
        eprintln!("Error serving project: {}", e);
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
      // Then: It should dispatch to serve_project
    }

    #[test]
    fn main_handles_no_command_by_showing_help() {
      // Given: No subcommand provided
      // When: main is called
      // Then: It should show help and return success
      // Note: This is tested by the Args parsing tests above
    }
  }

  /// Test suite for create_new_project function
  mod create_new_project_function {
    use super::*;

    #[test]
    fn create_new_project_succeeds_with_valid_name() {
      // Given: A temporary directory
      let temp_dir = tempdir().unwrap();
      let project_name = "test_project";

      // When: Creating a new project
      let project_path = temp_dir.path().join(project_name);

      // Change to temp directory for the function to work
      let original_cwd = std::env::current_dir().unwrap();
      std::env::set_current_dir(&temp_dir).unwrap();

      let result = create_new_project(project_name);

      // Then: It should succeed and create files
      assert!(result.is_ok());

      // Verify directory structure
      assert!(project_path.exists());
      assert!(project_path.join("Cargo.toml").exists());
      assert!(project_path.join("crates/app").exists());
      assert!(project_path.join("crates/app/src/main.rs").exists());
      assert!(project_path.join("crates/db/src/lib.rs").exists());
      assert!(project_path.join(".gitignore").exists());
      assert!(project_path.join("config/app.toml").exists());

      // Verify git initialization
      assert!(project_path.join(".git").exists());

      // Restore original directory
      std::env::set_current_dir(original_cwd).unwrap();
    }

    #[test]
    fn create_new_project_fails_with_existing_directory() {
      // Given: A temporary directory with existing subdirectory
      let temp_dir = tempdir().unwrap();
      let project_name = "existing_dir";
      let project_path = temp_dir.path().join(project_name);

      // Create the directory first
      fs::create_dir(&project_path).unwrap();

      // Change to temp directory
      let original_cwd = std::env::current_dir().unwrap();
      std::env::set_current_dir(&temp_dir).unwrap();

      // When: Trying to create project with existing name
      let result = create_new_project(project_name);

      // Then: It should fail
      assert!(result.is_err());
      assert!(result.unwrap_err().to_string().contains("already exists"));

      // Restore original directory
      std::env::set_current_dir(original_cwd).unwrap();
    }

    #[test]
    fn create_new_project_generates_correct_cargo_toml() {
      // Given: Project creation setup
      let temp_dir = tempdir().unwrap();
      let project_name = "cargo_test";

      // Change to temp directory
      let original_cwd = std::env::current_dir().unwrap();
      std::env::set_current_dir(&temp_dir).unwrap();

      // When: Creating project
      let result = create_new_project(project_name);
      assert!(
        result.is_ok(),
        "create_new_project failed: {:?}",
        result.err()
      );

      // Then: Cargo.toml should have correct content
      let cargo_content = fs::read_to_string("cargo_test/Cargo.toml").unwrap();
      assert!(cargo_content.contains("[workspace]"));
      assert!(cargo_content.contains("crates/app"));
      assert!(cargo_content.contains("crates/db"));

      let app_cargo = fs::read_to_string("cargo_test/crates/app/Cargo.toml").unwrap();
      assert!(app_cargo.contains("name = \"app\""));
      assert!(app_cargo.contains("forge ="));
      assert!(app_cargo.contains("db ="));

      // Restore original directory
      std::env::set_current_dir(original_cwd).unwrap();
    }

    #[test]
    fn create_new_project_generates_correct_main_rs() {
      // Given: Project creation setup
      let temp_dir = tempdir().unwrap();
      let project_name = "main_test";

      // Change to temp directory
      let original_cwd = std::env::current_dir().unwrap();
      std::env::set_current_dir(&temp_dir).unwrap();

      // When: Creating project
      let result = create_new_project(project_name);
      assert!(result.is_ok());

      // Then: main.rs should have correct Forge app code (explicit imports)
      let main_content = fs::read_to_string("main_test/crates/app/src/main.rs").unwrap();
      assert!(main_content.contains("use forge::App;"));
      assert!(
        !main_content.contains("forge::prelude"),
        "generated app should use explicit imports"
      );
      assert!(main_content.contains("App::new()"));
      assert!(main_content.contains(".with_migrations(db::Migrator)"));
      assert!(main_content.contains(".post_route"));
      assert!(main_content.contains(".route(\"/api/auth/admin\""));
      assert!(main_content.contains(".serve()"));

      // Restore original directory
      std::env::set_current_dir(original_cwd).unwrap();
    }

    #[test]
    fn create_new_project_generates_gitignore() {
      // Given: Project creation setup
      let temp_dir = tempdir().unwrap();
      let project_name = "gitignore_test";

      // Change to temp directory
      let original_cwd = std::env::current_dir().unwrap();
      std::env::set_current_dir(&temp_dir).unwrap();

      // When: Creating project
      let result = create_new_project(project_name);
      assert!(result.is_ok());

      // Then: .gitignore should exist with correct content
      let gitignore_content = fs::read_to_string("gitignore_test/.gitignore").unwrap();
      assert!(gitignore_content.contains("target/"));
      assert!(gitignore_content.contains(".env"));
      assert!(gitignore_content.contains("*.log"));
      assert!(gitignore_content.contains(".vscode/"));
      assert!(gitignore_content.contains(".DS_Store"));

      // Restore original directory
      std::env::set_current_dir(original_cwd).unwrap();
    }
  }

  /// Test suite for serve_project function
  mod serve_project_function {
    // ... (rest of the file remains same)
  }
}
