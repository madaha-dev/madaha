{
  projectRootFile = "flake.nix";
  programs.nixfmt.enable = true;
  programs.shellcheck.enable = true;
  programs.mdformat.enable = true;
  programs.yamlfmt.enable = true;

  settings.global.excludes = [
    "*.rasi"
    "*.jpg"
    "*.png"
    "Makefile"
    ".vscode/*"
    ".zed/*"
    ".opencode/*"
  ];
}
