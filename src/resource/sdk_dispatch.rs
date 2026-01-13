//! SDK Dispatch
//!
//! Maps SDK method names to actual OpenNebula API calls.

use crate::one::OneClient;
use anyhow::Result;
use serde_json::Value;

/// Invoke an SDK method
pub async fn invoke_sdk_method(
    service: &str,
    method: &str,
    client: &OneClient,
    params: &Value,
) -> Result<Value> {
    match service {
        "vm" => invoke_vm(method, client, params).await,
        "host" => invoke_host(method, client, params).await,
        "datastore" => invoke_datastore(method, client, params).await,
        "vnet" => invoke_vnet(method, client, params).await,
        "image" => invoke_image(method, client, params).await,
        "template" => invoke_template(method, client, params).await,
        "cluster" => invoke_cluster(method, client, params).await,
        "user" => invoke_user(method, client, params).await,
        "group" => invoke_group(method, client, params).await,
        "zone" => invoke_zone(method, client, params).await,
        "system" => invoke_system(method, client, params).await,
        _ => Err(anyhow::anyhow!("Unknown service: {}", service)),
    }
}

/// VM service methods
async fn invoke_vm(method: &str, client: &OneClient, params: &Value) -> Result<Value> {
    match method {
        "list" | "list_vms" => {
            let filter = params.get("filter").and_then(|v| v.as_i64()).unwrap_or(-2) as i32;
            let start = params.get("start").and_then(|v| v.as_i64()).unwrap_or(-1) as i32;
            let end = params.get("end").and_then(|v| v.as_i64()).unwrap_or(-1) as i32;
            let state = params.get("state").and_then(|v| v.as_i64()).unwrap_or(-1) as i32;
            client.list_vms(filter, start, end, state).await
        }
        "get" | "get_vm" => {
            let id = params
                .get("id")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow::anyhow!("Missing VM id"))? as i32;
            client.get_vm(id).await
        }
        "resume" => {
            let id = params
                .get("id")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow::anyhow!("Missing VM id"))? as i32;
            client.vm_action("resume", id).await
        }
        "suspend" => {
            let id = params
                .get("id")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow::anyhow!("Missing VM id"))? as i32;
            client.vm_action("suspend", id).await
        }
        "stop" => {
            let id = params
                .get("id")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow::anyhow!("Missing VM id"))? as i32;
            client.vm_action("stop", id).await
        }
        "poweroff" => {
            let id = params
                .get("id")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow::anyhow!("Missing VM id"))? as i32;
            client.vm_action("poweroff", id).await
        }
        "poweroff-hard" => {
            let id = params
                .get("id")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow::anyhow!("Missing VM id"))? as i32;
            client.vm_action("poweroff-hard", id).await
        }
        "reboot" => {
            let id = params
                .get("id")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow::anyhow!("Missing VM id"))? as i32;
            client.vm_action("reboot", id).await
        }
        "reboot-hard" => {
            let id = params
                .get("id")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow::anyhow!("Missing VM id"))? as i32;
            client.vm_action("reboot-hard", id).await
        }
        "terminate" => {
            let id = params
                .get("id")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow::anyhow!("Missing VM id"))? as i32;
            client.vm_action("terminate", id).await
        }
        "terminate-hard" => {
            let id = params
                .get("id")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow::anyhow!("Missing VM id"))? as i32;
            client.vm_action("terminate-hard", id).await
        }
        "undeploy" => {
            let id = params
                .get("id")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow::anyhow!("Missing VM id"))? as i32;
            client.vm_action("undeploy", id).await
        }
        "hold" => {
            let id = params
                .get("id")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow::anyhow!("Missing VM id"))? as i32;
            client.vm_action("hold", id).await
        }
        "release" => {
            let id = params
                .get("id")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow::anyhow!("Missing VM id"))? as i32;
            client.vm_action("release", id).await
        }
        _ => Err(anyhow::anyhow!("Unknown VM method: {}", method)),
    }
}

/// Host service methods
async fn invoke_host(method: &str, client: &OneClient, params: &Value) -> Result<Value> {
    match method {
        "list" | "list_hosts" => client.list_hosts().await,
        "get" | "get_host" => {
            let id = params
                .get("id")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow::anyhow!("Missing host id"))? as i32;
            client.get_host(id).await
        }
        _ => Err(anyhow::anyhow!("Unknown host method: {}", method)),
    }
}

/// Datastore service methods
async fn invoke_datastore(method: &str, client: &OneClient, params: &Value) -> Result<Value> {
    match method {
        "list" | "list_datastores" => client.list_datastores().await,
        "get" | "get_datastore" => {
            let id = params
                .get("id")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow::anyhow!("Missing datastore id"))?
                as i32;
            client.get_datastore(id).await
        }
        _ => Err(anyhow::anyhow!("Unknown datastore method: {}", method)),
    }
}

/// Virtual network service methods
async fn invoke_vnet(method: &str, client: &OneClient, params: &Value) -> Result<Value> {
    match method {
        "list" | "list_vnets" => {
            let filter = params.get("filter").and_then(|v| v.as_i64()).unwrap_or(-2) as i32;
            let start = params.get("start").and_then(|v| v.as_i64()).unwrap_or(-1) as i32;
            let end = params.get("end").and_then(|v| v.as_i64()).unwrap_or(-1) as i32;
            client.list_vnets(filter, start, end).await
        }
        "get" | "get_vnet" => {
            let id = params
                .get("id")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow::anyhow!("Missing vnet id"))? as i32;
            client.get_vnet(id).await
        }
        _ => Err(anyhow::anyhow!("Unknown vnet method: {}", method)),
    }
}

/// Image service methods
async fn invoke_image(method: &str, client: &OneClient, params: &Value) -> Result<Value> {
    match method {
        "list" | "list_images" => {
            let filter = params.get("filter").and_then(|v| v.as_i64()).unwrap_or(-2) as i32;
            let start = params.get("start").and_then(|v| v.as_i64()).unwrap_or(-1) as i32;
            let end = params.get("end").and_then(|v| v.as_i64()).unwrap_or(-1) as i32;
            client.list_images(filter, start, end).await
        }
        "get" | "get_image" => {
            let id = params
                .get("id")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow::anyhow!("Missing image id"))? as i32;
            client.get_image(id).await
        }
        _ => Err(anyhow::anyhow!("Unknown image method: {}", method)),
    }
}

/// Template service methods
async fn invoke_template(method: &str, client: &OneClient, params: &Value) -> Result<Value> {
    match method {
        "list" | "list_templates" => {
            let filter = params.get("filter").and_then(|v| v.as_i64()).unwrap_or(-2) as i32;
            let start = params.get("start").and_then(|v| v.as_i64()).unwrap_or(-1) as i32;
            let end = params.get("end").and_then(|v| v.as_i64()).unwrap_or(-1) as i32;
            client.list_templates(filter, start, end).await
        }
        "get" | "get_template" => {
            let id = params
                .get("id")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow::anyhow!("Missing template id"))? as i32;
            client.get_template(id).await
        }
        _ => Err(anyhow::anyhow!("Unknown template method: {}", method)),
    }
}

/// Cluster service methods
async fn invoke_cluster(method: &str, client: &OneClient, params: &Value) -> Result<Value> {
    match method {
        "list" | "list_clusters" => client.list_clusters().await,
        "get" | "get_cluster" => {
            let id = params
                .get("id")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow::anyhow!("Missing cluster id"))? as i32;
            client.get_cluster(id).await
        }
        _ => Err(anyhow::anyhow!("Unknown cluster method: {}", method)),
    }
}

/// User service methods
async fn invoke_user(method: &str, client: &OneClient, params: &Value) -> Result<Value> {
    match method {
        "list" | "list_users" => client.list_users().await,
        "get" | "get_user" => {
            let id = params
                .get("id")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow::anyhow!("Missing user id"))? as i32;
            client.get_user(id).await
        }
        _ => Err(anyhow::anyhow!("Unknown user method: {}", method)),
    }
}

/// Group service methods
async fn invoke_group(method: &str, client: &OneClient, _params: &Value) -> Result<Value> {
    match method {
        "list" | "list_groups" => client.list_groups().await,
        _ => Err(anyhow::anyhow!("Unknown group method: {}", method)),
    }
}

/// Zone service methods
async fn invoke_zone(method: &str, client: &OneClient, _params: &Value) -> Result<Value> {
    match method {
        "list" | "list_zones" => client.list_zones().await,
        _ => Err(anyhow::anyhow!("Unknown zone method: {}", method)),
    }
}

/// System service methods
async fn invoke_system(method: &str, client: &OneClient, _params: &Value) -> Result<Value> {
    match method {
        "version" | "get_version" => client.get_version().await,
        "config" | "get_config" => client.get_system_config().await,
        _ => Err(anyhow::anyhow!("Unknown system method: {}", method)),
    }
}
