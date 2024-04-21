//
// config.rs
// Copyright (C) 2024 Óscar García Amor <ogarcia@connectical.com>
// Distributed under terms of the GNU GPLv3 license.
//

use figment::{Figment, providers::{Env, Format, Serialized, Toml}};
use log::debug;

use crate::models::{Atlas, Config, Kubernetes};

const APP_NAME: &str = env!("CARGO_PKG_NAME");

impl Default for Config {
    fn default() -> Self {
        // By default all values are set to None
        let atlas = Atlas {
            public_key: None,
            private_key: None,
            clusters: None
        };
        let kubernetes = Kubernetes {
            kubeconfig: None,
            projects: None
        };
        Config {
            atlas,
            kubernetes
        }
    }
}

impl Config {
    pub fn figment(config_file: Option<String>) -> Figment {
        // Set a default config file in current directory
        let default_config_file = format!("{}.toml", APP_NAME);
        // Use paseed config file or default config file
        let config_file = match config_file {
            Some(config_file) => config_file,
            None => default_config_file
        };
        debug!("Config file location: {}", &config_file);
        Figment::from(Serialized::defaults(Config::default()))
            .merge(Toml::file(config_file))
            .merge(Env::prefixed(&format!("{}_", APP_NAME.to_uppercase())))
    }
}
