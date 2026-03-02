{
  inputs,
  pkgs,
  ...
}: {
  name = "{{PROJECT_NAME}}";

  languages = {
    javascript = {
      enable = true;
      bun.enable = true;
    };
    rust = {
      enable = true;
      channel = "stable";
      components = ["cargo" "rustc" "rustfmt" "rust-analyzer"];
    };
  };

  env = {
    RUST_BACKTRACE = "1";
    CARGO_TERM_COLOR = "always";
  };

  packages = with pkgs; [git];
}
