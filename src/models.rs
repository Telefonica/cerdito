//
// models.rs
// Copyright (C) 2024 Óscar García Amor <ogarcia@connectical.com>
// Distributed under terms of the GNU GPLv3 license.
//

use serde::{Deserialize, Serialize};

// Atlas clusters definition
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AtlasCluster {
    pub name: String,
    pub group_id: String
}

// Atlas definition
#[derive(Deserialize, Serialize)]
pub struct Atlas {
    pub public_key: Option<String>,
    pub private_key: Option<String>,
    pub clusters: Option<Vec<AtlasCluster>>
}

// AKS definition
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AKS {
    pub subscription_id: String,
    pub resource_group_name: String,
    pub resource_name: String
}

// Databricks definition
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Databricks {
    pub url: String,
    #[serde(default = "default_all_jobs")]
    pub all_jobs: bool,
    #[serde(default = "default_jobs")]
    pub jobs: Vec<String>,
    pub delete: Option<Vec<String>>
}

// Azure definition
#[derive(Deserialize, Serialize)]
pub struct Azure {
    pub tenant_id: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub aks: Option<Vec<AKS>>,
    pub databricks: Option<Vec<Databricks>>
}

// Kubernetes project definition
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KubernetesProject {
    pub namespace: String,
    pub deployments: Vec<String>
}

// Kubernetes definition
#[derive(Deserialize, Serialize)]
pub struct Kubernetes {
    pub kubeconfig: Option<String>,
    pub projects: Option<Vec<KubernetesProject>>
}

// cerdito main configuration
#[derive(Deserialize, Serialize)]
pub struct Config {
    pub atlas: Atlas,
    pub azure: Azure,
    pub kubernetes: Kubernetes
}

fn default_all_jobs() -> bool { false }
fn default_jobs() -> Vec<String> { std::vec::Vec::new() }
