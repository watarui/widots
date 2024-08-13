use lazy_static::lazy_static;
use std::collections::HashSet;

lazy_static! {
    pub static ref LINK_IGNORED_FILES: HashSet<&'static str> = {
        let mut s = HashSet::new();
        s.insert(".DS_Store");
        s.insert(".gitignore");
        s.insert("README.md");
        s
    };
    pub static ref LINK_IGNORED_PREFIXES: HashSet<&'static str> = {
        let mut s = HashSet::new();
        s.insert(".");
        s.insert("_");
        s
    };
    pub static ref LINK_IGNORED_ANCESTORS: HashSet<&'static str> = {
        let mut s = HashSet::new();
        s.insert(".git");
        s.insert("node_modules");
        s
    };
}

// pub const DEFAULT_CONFIG_PATH: &str = "~/.config/widots/config.yml";

pub const TEST_HOME_DIR: &str = ".widots-test";

// pub const BREW_FORMULA_FILENAME: &str = "brew_formulas.txt";
// pub const BREW_CASK_FORMULA_FILENAME: &str = "brew_casks.txt";

// pub const VSCODE_EXTENSIONS_FILENAME: &str = "vscode_extensions.txt";

// pub const FISH_COMPLETIONS_SOURCE_PATH: &str = "completions/widots.fish";
// pub const FISH_COMPLETIONS_TARGET_DIR: &str = ".config/fish/completions";
// pub const FISH_COMPLETIONS_FILENAME: &str = "widots.fish";
