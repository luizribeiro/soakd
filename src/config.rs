use crate::err::SprinklerError;
use serde::Deserialize;
use std::{fs, io::Read, path::Path};

#[derive(Deserialize, Clone)]
pub struct Configuration {
    pub mqtt: MQTTConfig,
    pub pump: PumpConfig,
    pub zones: Vec<ZoneConfig>,
    pub plans: Vec<SprinklerPlan>,
    #[serde(default = "default_driver")]
    pub driver: Driver,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Driver {
    Noop,
    Gpio,
}

#[derive(Deserialize, Clone)]
pub struct MQTTConfig {
    pub broker: String,
    pub port: u16,
    #[serde(default = "default_topic_prefix")]
    pub topic_prefix: String,
    #[serde(default = "default_client_id")]
    pub client_id: String,
}

fn default_topic_prefix() -> String {
    String::from("sprinklers")
}

fn default_client_id() -> String {
    String::from("soakd")
}

fn default_driver() -> Driver {
    Driver::Gpio
}

#[derive(Deserialize, Clone, Copy)]
pub struct PumpConfig {
    pub pin: u8,
    pub delay: u64,
}

#[derive(Deserialize, Clone)]
pub struct ZoneConfig {
    pub zone: String,
    pub pin: u8,
}

#[derive(Deserialize, Clone)]
pub struct SprinklerPlan {
    pub name: String,
    pub zone_durations: Vec<SprinklerZone>,
}

#[derive(Deserialize, Clone)]
pub struct SprinklerZone {
    pub zone: String,
    pub duration: u16,
}

pub fn read_config<P: AsRef<Path>>(path: P) -> Result<Configuration, SprinklerError> {
    let mut file = fs::File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config = serde_yaml::from_str(&contents)?;
    Ok(config)
}
