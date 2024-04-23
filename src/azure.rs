//
// azure.rs
// Copyright (C) 2024 Óscar García Amor <ogarcia@connectical.com>
// Distributed under terms of the GNU GPLv3 license.
//

use log::{debug, error, info, warn};
use serde::Deserialize;

use crate::models::{Azure, AKS};

const AZURE_URL: &str = "https://management.azure.com";
const AZURE_API_VERSION: &str = "2024-02-01";

const USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION")
);

#[derive(Deserialize)]
struct Token {
    access_token: String
}

impl Azure {
    pub fn new(tenant_id: Option<String>, client_id: Option<String>, client_secret: Option<String>, aks: Option<Vec<AKS>>) -> Azure {
        debug!("Azure tenant ID: {:?}", tenant_id);
        debug!("Azure client ID: {:?}", client_id);
        debug!("Azure client secret: {:?}", client_secret.as_ref().map(|k| "*".repeat(k.len())));
        debug!("AKS: {:?}", aks);
        Azure {
            tenant_id,
            client_id,
            client_secret,
            aks
        }
    }

    fn has_configuration(&self) -> bool {
        // Check if has the proper configuration
        if self.tenant_id.is_none() || self.client_id.is_none() || self.client_secret.is_none() {
            info!("No Azure credentials configured, skipping Azure action");
        } else if self.aks.is_none() {
            info!("No AKS configured, skipping Azure action");
        } else if self.tenant_id.clone().unwrap() == "" { // Safe to unwrap because was checked before
            warn!("Azure tenant ID cannot be an empty string, skipping Azure action");
        } else if self.client_id.clone().unwrap() == "" { // Safe to unwrap because was checked before
            warn!("Azure client ID cannot be an empty string, skipping Azure action");
        } else if self.client_secret.clone().unwrap() == "" { // Safe to unwrap because was checked before
            warn!("Azure client secret cannot be an empty string, skipping Azure action");
        } else {
            // The configurarion seems OK
            return true;
        }
        // Azure has not configured or configuration is wrong
        false
    }

    pub async fn pause(self, order: bool) {
        if self.has_configuration() {
            let (pre_action, action, post_action) = match order {
                true => ("Stopping", "stop", "stopped"),
                false => ("Starting", "start", "started")
            };
            let mut error = false;
            debug!("Trying to {} all configured AKS", action);
            // Create a http client
            let client = reqwest::Client::builder().connection_verbose(true).user_agent(USER_AGENT).build().expect("Client::new()");
            // Get the Azure token with values of self (safe to unwrap since has already been checked)
            let mut form_data = std::collections::HashMap::new();
            form_data.insert("grant_type", "client_credentials".to_string());
            form_data.insert("client_id", self.client_id.unwrap());
            form_data.insert("client_secret", self.client_secret.unwrap());
            form_data.insert("scope", format!("{AZURE_URL}/.default"));
            // Build URL
            let url = format!("https://login.microsoftonline.com/{}/oauth2/v2.0/token", self.tenant_id.unwrap());
            // Perform request
            let auth_response = client.post(&url)
                .form(&form_data)
                .send()
                .await;
            match auth_response {
                Ok(auth_response) => {
                    if auth_response.status().is_success() {
                        match auth_response.json::<Token>().await {
                            Ok(token) => {
                                for aks in self.aks.unwrap() {
                                    info!("{} AKS {}", &pre_action, &aks.resource_name);
                                    // Build action URL
                                    let url = format!("{AZURE_URL}/subscriptions/{}/resourceGroups/{}/providers/Microsoft.ContainerService/managedClusters/{}/{}?api-version={AZURE_API_VERSION}", &aks.subscription_id, &aks.resource_group_name, &aks.resource_name, &action);
                                    // Perform request
                                    let response = client.post(&url)
                                        .header("Authorization", format!("Bearer {}", token.access_token))
                                        .header("Content-Length", "0")
                                        .send()
                                        .await;
                                    match response {
                                        Ok(response) => {
                                            if response.status().is_success() {
                                                debug!("AKS {} {}", &aks.resource_name, &post_action);
                                            } else {
                                                let status = response.status();
                                                match response.text().await {
                                                    Ok(text) => if order && text.contains("is not currently running") {
                                                        info!("AKS {} is already paused", &aks.resource_name);
                                                    } else {
                                                        // Add text to empty text responses
                                                        let text = if &text == "" {
                                                            String::from("empty text response")
                                                        } else {
                                                            text
                                                        };
                                                        error!("Bad response status code {} when trying to {} AKS {}, {}", &status, &action, &aks.resource_name, &text);
                                                        error = true;
                                                    },
                                                    Err(err) => {
                                                        error!("Bad response status code {} when trying to {} AKS {}, {}", &status, &action, &aks.resource_name, &err);
                                                        error = true;
                                                    },
                                                }
                                            }
                                        },
                                        Err(err) => {
                                            error!("Unexpected response when trying to {} AKS {}, {}", &action, &aks.resource_name, &err);
                                            error = true;
                                        }
                                    }
                                }
                            },
                            Err(err) => {
                                error!("Cannot parse token from Azure response, {}", err);
                                error = true;
                            }
                        }
                    } else {
                        error!("Bad response status code {} when trying to obtain Azure token", auth_response.status());
                        error = true;
                    }
                },
                Err(err) => {
                    error!("Unexpected response when trying to obtain Azure token, {}", &err);
                    error = true;
                }
            }
            if error {
                debug!("Some (or all) AKS have failed to {}", &action)
            } else {
                debug!("All AKS have been {}", &post_action)
            }
        }
    }
}
