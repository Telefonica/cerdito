//
// azure.rs
// Copyright (C) 2024 Óscar García Amor <ogarcia@connectical.com>
// Distributed under terms of the GNU GPLv3 license.
//

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use crate::models::{Azure, AKS, Databricks};

const AZURE_URL: &str = "https://management.azure.com";
const AZURE_API_VERSION: &str = "2024-02-01";
const AZURE_DATABRICKS_SCOPE: &str = "2ff814a6-3304-4ab8-85cb-cd0e6f879c1d";

const USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION")
);

#[derive(Deserialize)]
struct Token {
    access_token: String
}

#[derive(Deserialize, Serialize)]
struct DatabricksJobSchedule {
    quartz_cron_expression: String,
    timezone_id: String,
    pause_status: String
}

#[derive(Deserialize)]
struct DatabricksJobSettings {
    name: String,
    schedule: Option<DatabricksJobSchedule>
}

#[derive(Deserialize)]
struct DatabricksJob {
    job_id: u64,
    settings: DatabricksJobSettings
}

#[derive(Deserialize)]
struct DatabricksJobs {
    jobs: Vec<DatabricksJob>
}

#[derive(Serialize)]
struct DatabricksJobUpdate {
    schedule: DatabricksJobSchedule
}

#[derive(Serialize)]
struct DatabricksJobUpdateRequest {
    job_id: u64,
    new_settings: DatabricksJobUpdate
}

impl Azure {
    pub fn new(tenant_id: Option<String>, client_id: Option<String>, client_secret: Option<String>, aks: Option<Vec<AKS>>, databricks: Option<Vec<Databricks>>) -> Azure {
        debug!("Azure tenant ID: {:?}", tenant_id);
        debug!("Azure client ID: {:?}", client_id);
        debug!("Azure client secret: {:?}", client_secret.as_ref().map(|k| "*".repeat(k.len())));
        debug!("AKS: {:?}", aks);
        debug!("Databricks: {:?}", databricks);
        Azure {
            tenant_id,
            client_id,
            client_secret,
            aks,
            databricks
        }
    }

    fn has_basic_configuration(&self) -> bool {
        // Check if has the proper basic configuration
        if self.tenant_id.is_none() || self.client_id.is_none() || self.client_secret.is_none() {
            info!("No Azure credentials configured, skipping Azure action");
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
        // No basic configuration or is wrong
        false
    }

    fn has_aks_configuration(&self) -> bool {
        // Check if has AKS configuration
        if self.aks.is_none() {
            info!("No AKS configured, skipping AKS action");
            false
        } else {
            true
        }
    }

    fn has_databricks_configuration(&self) -> bool {
        // Check if has Databricks configuration
        if self.databricks.is_none() {
            info!("No Databricks configured, skipping Databricks action");
            false
        } else {
            true
        }
    }

    async fn get_azure_token(&self, client: &reqwest::Client, scope: String) -> Result<String, reqwest::Error> {
        // Get the Azure token with values of self (safe to unwrap since has already been checked)
        let mut form_data = std::collections::HashMap::new();
        let grant_type = "client_credentials".to_string();
        form_data.insert("grant_type", &grant_type);
        form_data.insert("client_id", self.client_id.as_ref().unwrap());
        form_data.insert("client_secret", self.client_secret.as_ref().unwrap());
        form_data.insert("scope", &scope);
        // Build URL
        let url = format!("https://login.microsoftonline.com/{}/oauth2/v2.0/token", self.tenant_id.as_ref().unwrap());
        // Perform request
        let response = client.post(&url)
            .form(&form_data)
            .send()
            .await?;
        // Extract token from response
        Ok(response.error_for_status()?.json::<Token>().await.map(|t| t.access_token)?)
    }

    async fn get_databricks_jobs(&self, client: &reqwest::Client, token: &String, url: reqwest::Url) -> Result<DatabricksJobs, reqwest::Error> {
        // Get Databricks jobs list
        let response = client.get(url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Length", "0")
            .send()
            .await?;
        // Extract list from response
        Ok(response.error_for_status()?.json::<DatabricksJobs>().await?)
    }

    async fn pause_aks(&self, order: bool) {
        if self.has_aks_configuration() {
        let (pre_action, action, post_action) = match order {
                true => ("Stopping", "stop", "stopped"),
                false => ("Starting", "start", "started")
            };
            let mut error = false;
            debug!("Trying to {} all configured AKS", action);
            // Create a http client
            let client = reqwest::Client::builder().connection_verbose(true).user_agent(USER_AGENT).build().expect("Client::new()");
            let token = self.get_azure_token(&client, format!("{AZURE_URL}/.default")).await;
            match token {
                Ok(token) => {
                    for aks in self.aks.as_ref().unwrap() {
                        info!("{} AKS {}", &pre_action, &aks.resource_name);
                        // Build action URL
                        let url = format!("{AZURE_URL}/subscriptions/{}/resourceGroups/{}/providers/Microsoft.ContainerService/managedClusters/{}/{}?api-version={AZURE_API_VERSION}", &aks.subscription_id, &aks.resource_group_name, &aks.resource_name, &action);
                        // Perform request
                        let response = client.post(&url)
                            .header("Authorization", format!("Bearer {}", token))
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

    async fn pause_databricks(&self, order: bool) {
        if self.has_databricks_configuration() {
            let (pre_action, action, post_action, pause_status) = match order {
                true => ("Stopping", "stop", "stopped", "PAUSED"),
                false => ("Starting", "start", "started", "UNPAUSED")
            };
            let mut error = false;
            debug!("Trying to {} all configured Databricks", action);
            // Create a http client
            let client = reqwest::Client::builder().connection_verbose(true).user_agent(USER_AGENT).build().expect("Client::new()");
            let token = self.get_azure_token(&client, format!("{AZURE_DATABRICKS_SCOPE}/.default")).await;
            match token {
                Ok(token) => {
                    for databricks in self.databricks.as_ref().unwrap() {
                        info!("{} Databricks jobs in {}", &pre_action, &databricks.url);
                        // Build URL (Calling unwrap is safe in join because path is valid)
                        let url = reqwest::Url::parse(&databricks.url).map(|u| u.join("/api/2.1/jobs/list").unwrap());
                        match url {
                            Ok(url) => {
                                // Get Databricks jobs list to extract ID and schedule
                                match self.get_databricks_jobs(&client, &token, url).await {
                                    Ok(jobs) => {
                                        // Make list of configured jobs mutable to remove items
                                        let mut databricks_jobs = databricks.jobs.clone();
                                        for job in jobs.jobs {
                                            // Determine if job must be paused / unpaused
                                            let perform_action = if databricks.all_jobs && job.settings.schedule.is_some() {
                                                true
                                            } else if databricks_jobs.contains(&job.settings.name) {
                                                // Remove job from list
                                                databricks_jobs.retain(|j| *j != job.settings.name);
                                                if job.settings.schedule.is_none() {
                                                    warn!("It is not possible to {} job {} in {} because it is not scheduled in Databricks", &action, &job.settings.name, &databricks.url);
                                                    false
                                                } else {
                                                    true
                                                }
                                            } else {
                                                false
                                            };
                                            if perform_action {
                                                // Get schedule (Safe unwrap since is checked before) and change pause status
                                                let mut schedule = job.settings.schedule.unwrap();
                                                schedule.pause_status = pause_status.to_string();
                                                // Build URL (Safe unwrap since is checked before and path is valid)
                                                let url = reqwest::Url::parse(&databricks.url).map(|u| u.join("/api/2.1/jobs/update").unwrap()).unwrap();
                                                // Request change
                                                let json = DatabricksJobUpdateRequest {
                                                    job_id: job.job_id,
                                                    new_settings: DatabricksJobUpdate {
                                                        schedule: schedule
                                                    }
                                                };
                                                let response = client.post(url)
                                                    .header("Authorization", format!("Bearer {}", token))
                                                    .json(&json)
                                                    .send()
                                                    .await;
                                                match response {
                                                    Ok(response) => {
                                                        if response.status().is_success() {
                                                             info!("Job {} in {} {}", &job.settings.name, &databricks.url, &post_action);
                                                        } else {
                                                            error!("Bad response status code {} when trying to {} job {} in {}", response.status(), &action, &job.settings.name, &databricks.url);
                                                            error = true;
                                                        }
                                                    },
                                                    Err(err) => {
                                                        error!("Unexpected response when trying to {} job {} in {}, {}", &action, &job.settings.name, &databricks.url, &err);
                                                        error = true;
                                                    }
                                                }
                                            }
                                        }
                                        // If any configured job remains warn about it
                                        for job in databricks_jobs {
                                            warn!("It is not possible to {} job {} in {} because it is not defined in Databricks", &action, &job, &databricks.url)
                                        }
                                    },
                                    Err(err) => {
                                        error!("Error when trying to get Databricks jobs list, {}", &err);
                                        error = true;
                                    }
                                }
                            },
                            Err(err) => {
                                error!("Unexpected error when trying to parse Databricks URL, {}", &err);
                                error = true;
                            }
                        }
                    }
                },
                Err(err) => {
                    error!("Unexpected response when trying to obtain Azure token, {}", &err);
                    error = true;
                }
            }
            if error {
                debug!("Some (or all) Databricks have failed to {}", &action)
            } else {
                debug!("All Databricks have been {}", &post_action)
            }
        }
    }

    pub async fn pause(self, order: bool) {
        if self.has_basic_configuration() {
            self.pause_aks(order).await;
            self.pause_databricks(order).await;
        }
    }
}
