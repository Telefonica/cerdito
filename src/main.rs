//
// cerdito
// Copyright (C) 2024 Óscar García Amor <ogarcia@connectical.com>
// Distributed under terms of the GNU GPLv3 license.
//

use clap::{command, Arg, ArgAction, Command};
use env_logger::{Builder, Env};
use log::{info, LevelFilter};
use std::env;

mod atlas;
mod config;
mod kubernetes;
mod models;

use crate::models::{Atlas, Config, Kubernetes};

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
    // Main matches definition
    let matches = command!()
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .arg(Arg::new("config")
            .short('c')
            .long("config")
            .help("Custom configuration file path"))
        .arg(Arg::new("kubeconfig")
            .short('k')
            .long("kubeconfig")
            .help("Custom kubeconfig file path"))
        .arg(Arg::new("verbosity")
             .short('v')
             .long("verbose")
             .action(ArgAction::Count)
             .help("Sets the level of verbosity"))
        .subcommand(Command::new("start")
            .about("Start all configured elements"))
        .subcommand(Command::new("stop")
            .about("Stop all configured elements"))
        .subcommand(Command::new("version")
            .about("Prints version information"))
        .get_matches();

    // Configure loglevel
    match matches.get_count("verbosity") {
        0 => Builder::from_env(Env::default().filter_or(format!("{}_LOGLEVEL", APP_NAME.to_uppercase()), "off")).init(),
        1 => Builder::new().filter_level(LevelFilter::Info).init(),
        2 => Builder::new().filter_level(LevelFilter::Debug).init(),
        3 | _ => Builder::new().filter_level(LevelFilter::Trace).init()
    };
    info!("Log level: {:?}", log::max_level());

    // Get config file location from params or environment
    let config_file = matches.get_one::<String>("config").cloned().or(env::var(format!("{}_CONFIG", APP_NAME.to_uppercase())).ok());

    // Read config file
    let config = Config::figment(config_file);
    let config: Config = match config.extract() {
        Ok(config) => config,
        Err(err) => panic!("Error in config file: {}", err)
    };

    // Get Atlas keys from config file or environment
    let atlas_public_key = env::var("MONGODB_ATLAS_PUBLIC_KEY").ok().or(config.atlas.public_key);
    let atlas_private_key = env::var("MONGODB_ATLAS_PRIVATE_KEY").ok().or(config.atlas.private_key);

    // Configure Atlas client
    let atlas_client = Atlas::new(atlas_public_key, atlas_private_key, config.atlas.clusters);

    // Get Kubernetes config file location from params, environment or config
    let kubeconfig = matches.get_one::<String>("kubeconfig").cloned().or(env::var("KUBECONFIG").ok().or(config.kubernetes.kubeconfig));

    // Configure Kubernetes client
    let kubernetes_client = Kubernetes::new(kubeconfig, config.kubernetes.projects);

    match matches.subcommand() {
        Some(("version", _)) => println!("{} {}", APP_NAME, APP_VERSION),
        Some(("start", _)) => {
            atlas_client.pause(false).await;
            kubernetes_client.pause(false).await;
        },
        Some(("stop", _)) => {
            kubernetes_client.pause(true).await;
            atlas_client.pause(true).await;
        },
        _ => unreachable!()
    }
}
