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
    pub kubernetes: Kubernetes
}
