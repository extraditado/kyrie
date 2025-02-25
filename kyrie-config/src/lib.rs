use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, Clone)]
#[serde(default)]
pub struct KyrieConfig {}

impl KyrieConfig {
    pub fn load() -> Result<Self, ConfigError> {
        Ok(Self::default())
    }

    pub fn validate_config(&self) -> Result<(), Vec<String>> {
        self.validate().map_err(|e| {
            e.field_errors()
                .values()
                .flat_map(|errs| errs.iter().map(|e| e.to_string()))
                .collect::<Vec<String>>()
        })
    }
}

impl Default for KyrieConfig {
    fn default() -> Self {
        Self {}
    }
}

pub struct ConfigError;
