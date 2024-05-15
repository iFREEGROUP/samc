use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(default = "default_port")]
    pub server_port: u16,

    #[serde(default = "default_base_dir")]
    pub base_dir: String,

    pub use_api:Option<String>,

    #[serde(default = "default_tiny")]
    pub use_tiny:bool
}

fn default_tiny() -> bool {
    false
}

fn default_port() -> u16 {
    5050
}

fn default_base_dir() -> String {
    ".".to_string()
}