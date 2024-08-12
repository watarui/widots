use serde::Deserialize;
use std::path::PathBuf;

/// Represents the structure of the YAML configuration file.
#[derive(Debug, Deserialize, Default)]
pub struct Yaml {
    /// The list of links to create.
    #[serde(default)]
    pub link: Option<Vec<Link>>,
    /// The list of provisioning scripts to run.
    #[serde(default)]
    pub provision: Option<Vec<Provision>>,
}

/// Represents a link to be created.
#[derive(Debug, Deserialize)]
pub struct Link {
    /// The location of the file to link.
    pub location: PathBuf,
}

/// Represents a provisioning script to run.
#[derive(Debug, Deserialize)]
pub struct Provision {
    /// The mode in which to run the script.
    pub mode: String,
    /// The script to run.
    pub script: String,
}

impl Yaml {
    /// Creates a new `Yaml` instance.
    pub fn new() -> Self {
        Yaml::default()
    }
}

impl Link {
    /// Creates a new `Link` instance.
    ///
    /// # Arguments
    ///
    /// * `location` - The location of the file to link
    ///
    /// # Returns
    ///
    /// A new `Link` instance.
    pub fn new(location: PathBuf) -> Self {
        Link { location }
    }
}

impl Provision {
    /// Creates a new `Provision` instance.
    ///
    /// # Arguments
    ///
    /// * `mode` - The mode in which to run the script
    /// * `script` - The script to run
    ///
    /// # Returns
    ///
    /// A new `Provision` instance.
    pub fn new(mode: String, script: String) -> Self {
        Provision { mode, script }
    }
}
