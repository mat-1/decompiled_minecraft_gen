use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub java_args: Vec<String>,
}

impl Config {
    pub fn load() -> Self {
        let config_str = std::fs::read_to_string("config.toml").expect("couldn't read config.toml");
        toml::from_str(&config_str).expect("couldn't parse config.toml")
    }
}
