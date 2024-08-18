use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Config {
    #[serde(default)]
    pub link: Option<Vec<Link>>,
    #[serde(default)]
    pub provision: Option<Vec<Provision>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Link {
    pub location: PathBuf,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Provision {
    pub mode: String,
    pub script: String,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
        [[link]]
        location = "/path/to/dotfiles"

        [[provision]]
        mode = "macos"
        script = "echo 'Hello, macOS!'"

        [[provision]]
        mode = "linux"
        script = "echo 'Hello, Linux!'"
    "#;

        let config: Config = toml::from_str(toml_str).unwrap();

        assert_eq!(config.link.clone().unwrap().len(), 1);
        assert_eq!(config.provision.clone().unwrap().len(), 2);

        let link = &config.link.unwrap()[0];
        assert_eq!(link.location, PathBuf::from("/path/to/dotfiles"));

        let provisions = &config.provision.unwrap();
        assert_eq!(provisions[0].mode, "macos");
        assert_eq!(provisions[0].script, "echo 'Hello, macOS!'");
        assert_eq!(provisions[1].mode, "linux");
        assert_eq!(provisions[1].script, "echo 'Hello, Linux!'");
    }
}
