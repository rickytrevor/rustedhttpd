use serde::Serialize;
use serde::Deserialize;

use serde_yaml;

use std::fs::File;

use crate::structs::phpConnectionDetails;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerConfig {
    pub server: ServerSettings,
}


impl ServerConfig {
    pub fn get_server(&self) -> &ServerSettings {
        &self.server
    }

    pub fn get_php(&self) -> &Option<phpConnectionDetails> {
        &self.server.php
    }

    pub fn set_server(&mut self, server: ServerSettings) {
        self.server = server;
    }


}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerSettings {
    #[serde(default = "default_port")]
    pub port: u32,
    #[serde(default = "default_threads")]
    pub threads: usize,
    #[serde(default = "default_ttl")]
    pub ttl: u32,
    #[serde(default = "default_root")]
    pub root: String,
    #[serde(default = "default_php")]
    pub php: Option<phpConnectionDetails>,

}
 

fn default_php() -> Option<phpConnectionDetails> {
    println!("disabling php");
    None
}




fn default_port() -> u32 {
    println!("Unspecified port, using default: 8453");
    8453
}

fn default_threads() -> usize {
    println!("Unspecified threads, using default: 2");
    2
}

fn default_ttl() -> u32 {
    println!("Unspecified ttl, using default: 10");
    10
}

fn default_root() -> String {
    println!("Unspecified root, using default: ./web");
    "./web".to_string()
}

pub fn parse_config() -> ServerConfig {
    let file = File::open("./config.yaml").unwrap();
    let config: ServerConfig = serde_yaml::from_reader(file).unwrap();
    config
}
