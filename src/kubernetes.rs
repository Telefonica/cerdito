//
// kubernetes.rs
// Copyright (C) 2024 Óscar García Amor <ogarcia@connectical.com>
// Distributed under terms of the GNU GPLv3 license.
//

use k8s_openapi::api::apps::v1::Deployment;
use kube::{api::{Api, Patch, PatchParams}, config::{Kubeconfig, KubeConfigOptions}, Client, Config};
use log::{debug, error, info};

use crate::{models::{Kubernetes, KubernetesProject}, APP_NAME};

impl Kubernetes {
    pub fn new(kubeconfig: Option<String>, projects: Option<Vec<KubernetesProject>>) -> Kubernetes {
        debug!("Kubernetes kubeconfig file: {:?}", kubeconfig);
        debug!("Kubernetes projects: {:?}", projects);
        Kubernetes {
            kubeconfig,
            projects
        }
    }

    fn has_configuration(&self) -> bool {
        // Check if has the proper configuration
        if self.projects.is_none() {
            info!("No Kubernetes projects configured, skipping Kubernetes action");
        } else {
            // The configurarion seems OK
            return true;
        }
        // Kubernetes has not configured or configuration is wrong
        false
    }

    pub async fn pause(self, order: bool) {
        if self.has_configuration() {
            let (pre_action, action, post_action, replicas) = match order {
                true => ("Scaling down", "scale down", "scaled down", 0),
                false => ("Scaling up", "scale up", "scaled up", 1)
            };
            let mut error = false;
            debug!("Trying to {} all configured projects", action);
            // Read kubeconfig
            let kubeconfig = if self.kubeconfig.is_some() {
                // Use user defined kubeconfig location
                let kubeconfig = self.kubeconfig.unwrap();
                debug!("Using kubeconfig file {}", &kubeconfig);
                Kubeconfig::read_from(std::path::Path::new(&kubeconfig))
            } else {
                // Use default kubeconfig location
                debug!("Using default kubeconfig file location");
                Kubeconfig::read()
            };
            match kubeconfig {
                Ok(kubeconfig) => {
                    // Read config from kubeconfig
                    match Config::from_custom_kubeconfig(kubeconfig, &KubeConfigOptions::default()).await {
                        Ok(config) => {
                            // Create a k8s client
                            match Client::try_from(config) {
                                Ok(client) => {
                                    // Define the spec and params to scale deployments to desired replicas
                                    let patch = Patch::Merge(serde_json::json!({"spec": {"replicas": replicas}}));
                                    let params = PatchParams::apply(APP_NAME);
                                    for project in self.projects.unwrap() {
                                        info!("{} Kubernetes project {}", &pre_action, &project.namespace);
                                        // Manage deployments
                                        let deployments: Api<Deployment> = Api::namespaced(client.clone(), &project.namespace);
                                        for deployment in project.deployments {
                                            // Perform request
                                            match deployments.patch_scale(&deployment, &params, &patch).await {
                                                Ok(_) => info!("Kubernetes deployment {} {}", &deployment, &post_action),
                                                Err(err) => {
                                                    error!("Something has gone wrong with deployment {} {}, {}", &deployment, &action, err);
                                                    error = true;
                                                }
                                            }
                                        }
                                        if error {
                                            debug!("Some (or all) deploymens in Kubernetes project {} has failed to {}", &project.namespace, &action)
                                        } else {
                                            debug!("Kubernetes project {} have been {}", &project.namespace, &post_action)
                                        }
                                    }
                                    if error {
                                        debug!("Some (or all) projects have failed to {}", &action)
                                    } else {
                                        debug!("All projects have been {}", &post_action)
                                    }
                                },
                                Err(err) => error!("Kubernetes client cannot be configured, {}", err)
                            }
                        },
                        Err(err) => error!("There has been a problem reading config from kubeconfig, {}", err)
                    }
                },
                Err(err) => error!("There has been a problem with kubeconfig, {}", err)
            }
        }
    }
}
