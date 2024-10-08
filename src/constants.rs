use std::collections::HashSet;
use std::sync::LazyLock;

pub static LINK_IGNORED_FILES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let mut s = HashSet::new();
    s.insert(".DS_Store");
    s.insert(".gitignore");
    s
});

pub static LINK_IGNORED_PREFIXES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let mut s = HashSet::new();
    s.insert("_");
    s
});

pub static LINK_IGNORED_ANCESTORS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let mut s = HashSet::new();
    s.insert(".git");
    s.insert("node_modules");
    s
});

pub const APP_NAME: &str = "widots";
pub const DEFAULT_CONFIG_TOML: &str = "~/.config/widots/config.toml";

pub const TEST_HOME_DIR: &str = ".widots-test";

pub const BREW_FORMULA_FILENAME: &str = "brew_formulas.txt";
pub const BREW_CASK_FORMULA_FILENAME: &str = "brew_casks.txt";

pub const RESOURCES_DIR: &str = "resources";
pub const VSCODE_EXTENSIONS_FILENAME: &str = "vscode_extensions.txt";

pub const DEPLOY_SOURCE_PATH: &str = "target/release/widots";
pub const DEPLOY_DESTINATION_PATH: &str = "/usr/local/bin/widots";

pub const FISH_COMPLETIONS_SOURCE_PATH: &str = "completions/widots.fish";
pub const FISH_COMPLETIONS_TARGET_DIR: &str = "~/.config/fish/completions";
pub const FISH_COMPLETIONS_FILENAME: &str = "widots.fish";
