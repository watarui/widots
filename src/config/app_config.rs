use crate::core::os::OSOperations;
use crate::core::path::PathOperations;
use crate::core::shell::ShellOperations;
use crate::utils::yaml::YamlOperations;
use std::sync::Arc;

pub struct AppConfig {
    pub os_detector: Arc<dyn OSOperations>,
    pub path_expander: Arc<dyn PathOperations>,
    pub shell_executor: Arc<dyn ShellOperations>,
    pub yaml_parser: Arc<dyn YamlOperations>,
}

impl AppConfig {
    pub fn new(
        os_detector: Arc<dyn OSOperations>,
        path_expander: Arc<dyn PathOperations>,
        shell_executor: Arc<dyn ShellOperations>,
        yaml_parser: Arc<dyn YamlOperations>,
    ) -> Self {
        Self {
            os_detector,
            path_expander,
            shell_executor,
            yaml_parser,
        }
    }
}
