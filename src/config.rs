use std::env::var;
use custom_error::custom_error;
use std::clone::Clone;
use std::fs::File;
use std::io::{Read, ErrorKind};
use std::io::Error as IOError;
use toml::de::Error as TomlError;
use std::collections::HashMap;

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
    pub bind_address: Option<String>,
    pub port: Option<u16>,
    pub caching: HashMap<String, Caching>,
    pub buckets: HashMap<String, BucketConfiguration>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BucketConfiguration {
    pub host: String,
    pub bucket: Option<String>,
    pub index: Option<String>,
    pub not_found: Option<String>,
    pub cache_name: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Caching {
    #[serde(rename="type")]
    pub caching_type: Option<String>,
    pub ttl: Option<u64>,

    // local cache
    pub capacity: Option<usize>,

    // redis
    pub host: Option<String>,
    pub port: Option<u16>,
}

impl Caching {
    pub fn override_with(&self, other: &Caching) -> Caching {
        let mut this_clone = self.clone();
        let other = other.clone();

        if other.caching_type.is_some() {
            this_clone.caching_type = other.caching_type;
        }

        if other.capacity.is_some() {
            this_clone.capacity = other.capacity;
        }

        if other.ttl.is_some() {
            this_clone.ttl = other.ttl;
        }

        this_clone
    }
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