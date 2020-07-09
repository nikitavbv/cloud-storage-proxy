use std::env::var;
use custom_error::custom_error;
use std::clone::Clone;
use std::fs::File;
use std::io::{Read, ErrorKind};
use std::io::Error as IOError;
use toml::de::Error as TomlError;
use std::collections::HashMap;
use std::iter::Cloned;
use std::rc::Rc;
use std::sync::Arc;

custom_error! {pub LoadConfigError
    FailedToRead{source: IOError} = "failed to read config file: {source}",
    FailedToDeserialize{source: TomlError} = "failed to deserialize config: {source}"
}

impl From<LoadConfigError> for IOError {
    fn from(load_config_error: LoadConfigError) -> Self {
        IOError::new(ErrorKind::Other, format!("{}", load_config_error))
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub service_account_key: Option<String>,
    pub service_account_key_file: Option<String>,
    pub buckets: HashMap<String, BucketConfiguration>
}

#[derive(Deserialize, Debug, Clone)]
pub struct BucketConfiguration {
    pub host: String,
    pub bucket: Option<String>
}

impl Config {

    pub fn bucket_configuration_by_host(&self, host: &str) -> Option<&BucketConfiguration> {
        self.buckets.values().find(|v| v.host  == host)
    }
}

fn get_config_file_name() -> String {
    var("CONFIG_FILE").unwrap_or("config.toml".into())
}

pub fn load_config() -> Result<Config, LoadConfigError> {
    let mut config_file = File::open(get_config_file_name())
        .map_err(|source| LoadConfigError::FailedToRead { source })?;
    let mut config_str = String::new();
    config_file.read_to_string(&mut config_str)
        .map_err(|source| LoadConfigError::FailedToRead { source })?;

    toml::from_str(&config_str).map_err(|source| LoadConfigError::FailedToDeserialize { source })
}