use ron::ser::{to_string_pretty};
use config::{Config as ConfigLoader, File};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub mod default;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpeedPoint {
    pub temp: f32,
    pub speed: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Strategy {
    pub fan_speed_update_frequency: f32,
    pub moving_average_interval: u32,
    pub speed_curve: Vec<SpeedPoint>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FanConfig {
    pub default_strategy: String,
    pub strategy_on_discharging: String,
    pub strategies: std::collections::HashMap<String, Strategy>,
}

fn get_config_file() -> PathBuf {
    let mut path = PathBuf::from("/etc/fw-fanctrl-rs");
    fs::create_dir_all(&path).unwrap();
    path.push("config.ron");
    path
}

fn write_config<P: AsRef<Path>>(path: P, config: &FanConfig) -> std::io::Result<()> {
    let ron_string = to_string_pretty(config, ron::ser::PrettyConfig::default())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    fs::write(path, ron_string)
}

pub fn load_or_create_config() -> Result<FanConfig, Box<dyn std::error::Error>> {
    let path = get_config_file();
    if !path.exists() {
        let default = default::default_fan_config();
        write_config(&path, &default)?;
        return Ok(default);
    }

    let loaded = ConfigLoader::builder()
        .add_source(File::with_name(path.to_str().unwrap()))
        .build()?
        .try_deserialize::<FanConfig>()?;

    Ok(loaded)
}
