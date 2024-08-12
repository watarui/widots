pub const BREW_FORMULA_FILENAME: &str = "formula.txt";
pub const BREW_CASK_FORMULA_FILENAME: &str = "cask-formula.txt";
pub const APP_RESOURCE_DIR: &str = "resources";

pub const DEPLOY_SOURCE_PATH: &str = "target/release/widots";
pub const DEPLOY_DESTINATION_PATH: &str = "/usr/local/bin/widots";

pub const LINK_TEST_DIR: &str = ".widots_test";

pub const VSCODE_EXTENSIONS_FILENAME: &str = "vscode-extensions.txt";

pub const FISH_COMPLETIONS_SOURCE_PATH: &str = "completions/widots.fish";
pub const FISH_COMPLETIONS_TARGET_DIR: &str = ".config/fish/completions";
pub const FISH_COMPLETIONS_FILENAME: &str = "widots.fish";

pub const LINK_IGNORED_FILES: [&str; 4] =
    [".DS_Store", ".fseventsd", ".Spotlight-V100", ".Trashes"];
pub const LINK_IGNORED_PREFIXES: [&str; 1] = ["._"];
pub const LINK_IGNORED_ANCESTORS: [&str; 2] = [".git", "node_modules"];
pub const LINK_IGNORED_GIT_FILES: [&str; 2] = ["config", "ignore"];
