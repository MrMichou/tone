//! OpenNebula Client
//!
//! Main client for interacting with OpenNebula's XML-RPC API.

use super::auth::OneCredentials;
use super::xmlrpc::{build_method_call, parse_one_xml_to_json, parse_response, XmlRpcResponse, XmlRpcValue};
use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;

/// Main OpenNebula client
#[derive(Clone)]
pub struct OneClient {
    pub credentials: OneCredentials,
    http: Client,
}

impl OneClient {
    /// Create a new OpenNebula client
    pub async fn new() -> Result<Self> {
        let credentials = OneCredentials::new()?;

        let http = Client::builder()
            .user_agent("tone/0.1.0")
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { credentials, http })
    }

    /// Create a new client with custom endpoint
    pub async fn with_endpoint(endpoint: &str) -> Result<Self> {
        let mut credentials = OneCredentials::new()?;
        credentials.endpoint = endpoint.to_string();

        let http = Client::builder()
            .user_agent("tone/0.1.0")
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { credentials, http })
    }

    /// Make an XML-RPC call to OpenNebula
    pub async fn call(&self, method: &str, params: Vec<XmlRpcValue>) -> Result<Value> {
        // Prepend auth string to params
        let mut full_params = vec![XmlRpcValue::String(self.credentials.auth_string())];
        full_params.extend(params);

        let xml_request = build_method_call(method, &full_params)?;

        tracing::debug!("XML-RPC call: {} to {}", method, self.credentials.endpoint);
        tracing::trace!("Request XML: {}", xml_request);

        let response = self
            .http
            .post(&self.credentials.endpoint)
            .header("Content-Type", "text/xml")
            .body(xml_request)
            .send()
            .await
            .context("Failed to send XML-RPC request")?;

        let status = response.status();
        let body = response
            .text()
            .await
            .context("Failed to read response body")?;

        if !status.is_success() {
            tracing::error!("HTTP error: {} - {}", status, body);
            return Err(anyhow::anyhow!("HTTP request failed: {} - {}", status, body));
        }

        tracing::trace!("Response XML: {}", body);

        let parsed = parse_response(&body)?;

        match parsed {
            XmlRpcResponse::Success(value) => {
                // OpenNebula returns an array [success, data, error_code]
                if let XmlRpcValue::Array(arr) = value {
                    if arr.len() >= 2 {
                        // First element is boolean success
                        let success = match &arr[0] {
                            XmlRpcValue::Boolean(b) => *b,
                            _ => true,
                        };

                        if !success {
                            // Second element contains error message
                            let error_msg = match &arr[1] {
                                XmlRpcValue::String(s) => s.clone(),
                                _ => "Unknown error".to_string(),
                            };
                            return Err(anyhow::anyhow!("OpenNebula API error: {}", error_msg));
                        }

                        // Second element contains the data (usually XML string)
                        match &arr[1] {
                            XmlRpcValue::String(xml_data) => {
                                // Parse the XML data to JSON
                                parse_one_xml_to_json(xml_data)
                            }
                            XmlRpcValue::Int(i) => Ok(Value::Number((*i).into())),
                            other => Ok(super::xmlrpc::xmlrpc_to_json(other)),
                        }
                    } else {
                        Ok(Value::Array(
                            arr.iter().map(super::xmlrpc::xmlrpc_to_json).collect(),
                        ))
                    }
                } else {
                    Ok(super::xmlrpc::xmlrpc_to_json(&value))
                }
            }
            XmlRpcResponse::Fault(fault) => {
                let msg = format!("XML-RPC fault: {:?}", fault);
                Err(anyhow::anyhow!(msg))
            }
        }
    }

    // =========================================================================
    // VM Pool API
    // =========================================================================

    /// List all VMs (one.vmpool.info)
    /// filter: -2 = all, -1 = mine, >= 0 = specific user
    /// start/end: -1 = all
    /// state: -1 = all, or specific state filter
    pub async fn list_vms(&self, filter: i32, start: i32, end: i32, state: i32) -> Result<Value> {
        self.call(
            "one.vmpool.info",
            vec![
                XmlRpcValue::Int(filter),
                XmlRpcValue::Int(start),
                XmlRpcValue::Int(end),
                XmlRpcValue::Int(state),
            ],
        )
        .await
    }

    /// Get VM info (one.vm.info)
    pub async fn get_vm(&self, vm_id: i32) -> Result<Value> {
        self.call("one.vm.info", vec![XmlRpcValue::Int(vm_id)]).await
    }

    /// Perform VM action (one.vm.action)
    pub async fn vm_action(&self, action: &str, vm_id: i32) -> Result<Value> {
        self.call(
            "one.vm.action",
            vec![XmlRpcValue::String(action.to_string()), XmlRpcValue::Int(vm_id)],
        )
        .await
    }

    // =========================================================================
    // Host Pool API
    // =========================================================================

    /// List all hosts (one.hostpool.info)
    pub async fn list_hosts(&self) -> Result<Value> {
        self.call("one.hostpool.info", vec![]).await
    }

    /// Get host info (one.host.info)
    pub async fn get_host(&self, host_id: i32) -> Result<Value> {
        self.call("one.host.info", vec![XmlRpcValue::Int(host_id)])
            .await
    }

    // =========================================================================
    // Datastore Pool API
    // =========================================================================

    /// List all datastores (one.datastorepool.info)
    pub async fn list_datastores(&self) -> Result<Value> {
        self.call("one.datastorepool.info", vec![]).await
    }

    /// Get datastore info (one.datastore.info)
    pub async fn get_datastore(&self, ds_id: i32) -> Result<Value> {
        self.call("one.datastore.info", vec![XmlRpcValue::Int(ds_id)])
            .await
    }

    // =========================================================================
    // Virtual Network Pool API
    // =========================================================================

    /// List all virtual networks (one.vnpool.info)
    /// filter: -2 = all, -1 = mine, >= 0 = specific user
    pub async fn list_vnets(&self, filter: i32, start: i32, end: i32) -> Result<Value> {
        self.call(
            "one.vnpool.info",
            vec![
                XmlRpcValue::Int(filter),
                XmlRpcValue::Int(start),
                XmlRpcValue::Int(end),
            ],
        )
        .await
    }

    /// Get virtual network info (one.vn.info)
    pub async fn get_vnet(&self, vnet_id: i32) -> Result<Value> {
        self.call("one.vn.info", vec![XmlRpcValue::Int(vnet_id)])
            .await
    }

    // =========================================================================
    // Image Pool API
    // =========================================================================

    /// List all images (one.imagepool.info)
    /// filter: -2 = all, -1 = mine, >= 0 = specific user
    pub async fn list_images(&self, filter: i32, start: i32, end: i32) -> Result<Value> {
        self.call(
            "one.imagepool.info",
            vec![
                XmlRpcValue::Int(filter),
                XmlRpcValue::Int(start),
                XmlRpcValue::Int(end),
            ],
        )
        .await
    }

    /// Get image info (one.image.info)
    pub async fn get_image(&self, image_id: i32) -> Result<Value> {
        self.call("one.image.info", vec![XmlRpcValue::Int(image_id)])
            .await
    }

    // =========================================================================
    // Template Pool API
    // =========================================================================

    /// List all templates (one.templatepool.info)
    /// filter: -2 = all, -1 = mine, >= 0 = specific user
    pub async fn list_templates(&self, filter: i32, start: i32, end: i32) -> Result<Value> {
        self.call(
            "one.templatepool.info",
            vec![
                XmlRpcValue::Int(filter),
                XmlRpcValue::Int(start),
                XmlRpcValue::Int(end),
            ],
        )
        .await
    }

    /// Get template info (one.template.info)
    pub async fn get_template(&self, template_id: i32) -> Result<Value> {
        self.call("one.template.info", vec![XmlRpcValue::Int(template_id)])
            .await
    }

    // =========================================================================
    // Cluster Pool API
    // =========================================================================

    /// List all clusters (one.clusterpool.info)
    pub async fn list_clusters(&self) -> Result<Value> {
        self.call("one.clusterpool.info", vec![]).await
    }

    /// Get cluster info (one.cluster.info)
    pub async fn get_cluster(&self, cluster_id: i32) -> Result<Value> {
        self.call("one.cluster.info", vec![XmlRpcValue::Int(cluster_id)])
            .await
    }

    // =========================================================================
    // User Pool API
    // =========================================================================

    /// List all users (one.userpool.info)
    pub async fn list_users(&self) -> Result<Value> {
        self.call("one.userpool.info", vec![]).await
    }

    /// Get user info (one.user.info)
    pub async fn get_user(&self, user_id: i32) -> Result<Value> {
        self.call("one.user.info", vec![XmlRpcValue::Int(user_id)])
            .await
    }

    // =========================================================================
    // Group Pool API
    // =========================================================================

    /// List all groups (one.grouppool.info)
    pub async fn list_groups(&self) -> Result<Value> {
        self.call("one.grouppool.info", vec![]).await
    }

    // =========================================================================
    // Zone API
    // =========================================================================

    /// List all zones (one.zonepool.info)
    pub async fn list_zones(&self) -> Result<Value> {
        self.call("one.zonepool.info", vec![]).await
    }

    // =========================================================================
    // System API
    // =========================================================================

    /// Get OpenNebula version (one.system.version)
    pub async fn get_version(&self) -> Result<Value> {
        self.call("one.system.version", vec![]).await
    }

    /// Get system config (one.system.config)
    pub async fn get_system_config(&self) -> Result<Value> {
        self.call("one.system.config", vec![]).await
    }
}

/// Format an OpenNebula API error for display
pub fn format_one_error(error: &anyhow::Error) -> String {
    let error_str = error.to_string();

    // Clean up common error patterns
    if error_str.contains("401") || error_str.contains("Authentication") {
        return "Authentication failed. Check ONE_AUTH credentials.".to_string();
    }
    if error_str.contains("Connection refused") {
        return "Connection refused. Check ONE_XMLRPC endpoint.".to_string();
    }
    if error_str.contains("timeout") {
        return "Request timed out. Server may be unreachable.".to_string();
    }

    // Truncate long error messages
    if error_str.len() > 100 {
        format!("{}...", &error_str[..100])
    } else {
        error_str
    }
}
