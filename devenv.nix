{
  inputs,
  pkgs,
  ...
}: {
  # Name of the project with version
  name = "forge";

  # Languages
  languages = {
    javascript = {
      enable = true;
      bun = {
        enable = true;
      };
    };

    rust = {
      enable = true;
      channel = "stable";
      components = [
        "cargo"
        "clippy"
        "rust-analyzer"
        "rustc"
        "rustfmt"
        "llvm-tools"
      ];
      targets = [];
    };
  };

  env = {
    RUST_BACKTRACE = "1";
    CARGO_TERM_COLOR = "always";
    # Enable SQL statement logging (forge db layer). SQL appears when FORGE_SQL_DEBUG=1 and sqlx=debug below.
    FORGE_SQL_DEBUG = "1";
    # Show SQL queries (sqlx) and app/forge at debug. Omit sqlx=debug to disable SQL logging.
    RUST_LOG = "info,forge=debug,app=debug,sqlx=debug";
  };

  # Development packages
  packages = with pkgs; [
    # AI
    inputs.nixpkgs-unstable.legacyPackages.${stdenv.hostPlatform.system}.beads

    # E2E browser automation (fantoccini + chromedriver)
    chromedriver
    chromium

    # Rust tools
    clippy
    rust-analyzer
    rustc

    # Development tools
    direnv
    # Git hooks (prek = pre-commit replacement, single binary, no Python)
    prek

    # Formatting tools
    alejandra

    # Publishing tools
    cargo-watch
    cargo-audit
    cargo-llvm-cov
    cargo-nextest

    # Version management
    git
    gh

    # treefmt
    actionlint
    alejandra
    beautysh
    biome
    deadnix
    rustfmt
    taplo
    treefmt
    vulnix
    yamlfmt
  ];

  scripts = {
    prek-install = {
      exec = ''
        prek install -q --overwrite
      '';
    };
  };

  enterShell = ''
    prek-install

    # Add forge CLI to PATH if it exists, prioritizing debug during dev
    if [ -f ./target/debug/forge ] && [ -f ./target/release/forge ]; then
      if [ ./target/debug/forge -nt ./target/release/forge ]; then
        export PATH="$PWD/target/debug:$PATH"
        echo "Forge CLI (debug) available in PATH"
      else
        export PATH="$PWD/target/release:$PATH"
        echo "Forge CLI (release) available in PATH"
      fi
    elif [ -f ./target/debug/forge ]; then
      export PATH="$PWD/target/debug:$PATH"
      echo "Forge CLI (debug) available in PATH"
    elif [ -f ./target/release/forge ]; then
      export PATH="$PWD/target/release:$PATH"
      echo "Forge CLI (release) available in PATH"
    fi
  '';
}
