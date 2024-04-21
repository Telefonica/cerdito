//
// atlas.rs
// Copyright (C) 2024 Óscar García Amor <ogarcia@connectical.com>
// Distributed under terms of the GNU GPLv3 license.
//

use diqwest::WithDigestAuth;
use log::{debug, error, info, warn};
use serde::Serialize;

use crate::models::{Atlas, AtlasCluster};

const ATLAS_URL: &str = "https://cloud.mongodb.com";
const ATLAS_API_VERSION: &str = "v2";

#[derive(Serialize)]
struct Pause {
    paused: bool
}

impl Atlas {
    pub fn new(public_key: Option<String>, private_key: Option<String>, clusters: Option<Vec<AtlasCluster>>) -> Atlas {
        debug!("Atlas public key: {:?}", public_key);
        debug!("Atlas private key: {:?}", private_key.as_ref().map(|k| "*".repeat(k.len())));
        debug!("Atlas clusters: {:?}", clusters);
        Atlas {
            public_key,
            private_key,
            clusters
        }
    }

    fn has_configuration(&self) -> bool {
        // Check if has the proper configuration
        if self.public_key.is_none() || self.private_key.is_none() {
            info!("No Atlas credentials configured, skipping Atlas action");
        } else if self.clusters.is_none() {
            info!("No Atlas clusters configured, skipping Atlas action");
        } else if self.public_key.clone().unwrap() == "" { // Safe to unwrap because was checked before
            warn!("Atlas public key cannot be an empty string, skipping Atlas action");
        } else if self.private_key.clone().unwrap() == "" { // Safe to unwrap because was checked before
            warn!("Atlas private key cannot be an empty string, skipping Atlas action");
        } else {
            // The configurarion seems OK
            return true;
        }
        // Atlas has not configured or configuration is wrong
        false
    }

    pub async fn pause(self, order: bool) {
        if self.has_configuration() {
            let (pre_action, action, post_action) = match order {
                true => ("Stopping", "stop", "stopped"),
                false => ("Starting", "start", "started")
            };
            let mut error = false;
            debug!("Trying to {} all configured clusters", action);
            // Create a http client
            let client = reqwest::Client::builder().connection_verbose(true).build().expect("Client::new()");
            // Get values from self (safe to unwrap since has already been checked)
            let public_key = self.public_key.unwrap();
            let private_key = self.private_key.unwrap();
            for cluster in self.clusters.unwrap() {
                info!("{} Atlas cluster {}", &pre_action, &cluster.name);
                // Build URL
                let url = format!("{}/api/atlas/{}/groups/{}/clusters/{}", &ATLAS_URL, &ATLAS_API_VERSION, &cluster.group_id, &cluster.name);
                // Perform request
                let response = client.patch(&url)
                    .header("accept", "application/vnd.atlas.2023-02-01+json")
                    .json(&Pause{paused: order})
                    .send_with_digest_auth(&public_key, &private_key)
                    .await;
                match response {
                    Ok(response) => {
                        if response.status().is_success() {
                            debug!("Atlas cluster {} {}", &cluster.name, &post_action)
                        } else {
                            let status = response.status();
                            match response.text().await {
                                Ok(text) => if text.contains("CLUSTER_ALREADY_PAUSED") {
                                    info!("Atlas cluster {} is already paused", &cluster.name);
                                } else {
                                    // Add text to empty text responses
                                    let text = if &text == "" {
                                        String::from("empty text response")
                                    } else {
                                        text
                                    };
                                    error!("Bad response status code {} when trying to {} cluster {}, {}", &status, &action, &cluster.name, &text);
                                    error = true;
                                },
                                Err(err) => {
                                    error!("Bad response status code {} when trying to {} cluster {}, {}", &status, &action, &cluster.name, &err);
                                    error = true;
                                }
                            }
                        }
                    },
                    Err(err) => {
                        error!("Unexpected response when trying to {} cluster {}, {}", &action, &cluster.name, &err);
                        error = true;
                    }
                }
            }
            if error {
                debug!("Some (or all) clusters have failed to {}", &action)
            } else {
                debug!("All clusters have been {}", &post_action)
            }
        }
    }
}
