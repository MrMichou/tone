//! OpenNebula Client
//!
//! Main client for interacting with OpenNebula's XML-RPC API.
//!
//! Security features:
//! - Connection timeouts to prevent DoS
//! - Credentials are never logged (redacted in trace output)
//! - Uses secure credential handling from auth module

use super::auth::OneCredentials;
use super::xmlrpc::{
    build_method_call, parse_one_xml_to_json, parse_response, XmlRpcResponse, XmlRpcValue,
};
use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use zeroize::Zeroize;

/// Default timeout for HTTP requests (30 seconds)
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Main OpenNebula client
#[derive(Clone)]
pub struct OneClient {
    credentials: OneCredentials,
    http: Client,
}

impl OneClient {
    /// Create a new OpenNebula client
    pub async fn new() -> Result<Self> {
        let credentials = OneCredentials::new()?;

        let http = Client::builder()
            .user_agent("tone/0.1.0")
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { credentials, http })
    }

    /// Create a new client with custom endpoint
    pub async fn with_endpoint(endpoint: &str) -> Result<Self> {
        let mut credentials = OneCredentials::new()?;
        credentials.set_endpoint(endpoint.to_string());

        let http = Client::builder()
            .user_agent("tone/0.1.0")
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { credentials, http })
    }

    /// Create a client for testing with explicit credentials (no file/env lookup)
    #[cfg(test)]
    pub fn for_testing(endpoint: &str) -> Self {
        use super::auth::OneCredentials;
        let credentials = OneCredentials::for_testing("test", "test", endpoint);
        let http = Client::builder()
            .user_agent("tone/0.1.0-test")
            .timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to create HTTP client for testing");
        Self { credentials, http }
    }

    /// Get the endpoint URL (for display purposes)
    pub fn endpoint(&self) -> &str {
        self.credentials.endpoint()
    }

    /// Get the username (for display purposes)
    pub fn username(&self) -> &str {
        self.credentials.username()
    }

    /// Maximum response body size (10 MB) to prevent OOM from malicious servers
    const MAX_RESPONSE_SIZE: usize = 10 * 1024 * 1024;

    /// Make an XML-RPC call to OpenNebula
    pub async fn call(&self, method: &str, params: Vec<XmlRpcValue>) -> Result<Value> {
        // Prepend auth string to params
        let mut auth_string = self.credentials.auth_string();
        let mut full_params = vec![XmlRpcValue::String(auth_string.clone())];
        auth_string.zeroize();
        full_params.extend(params);

        let mut xml_request = build_method_call(method, &full_params)?;
        // Drop full_params early to zeroize the auth copy inside XmlRpcValue
        drop(full_params);

        tracing::debug!(
            "XML-RPC call: {} to {}",
            method,
            self.credentials.endpoint()
        );
        // SECURITY: Never log the actual request XML as it contains credentials
        tracing::trace!(
            "Request XML: [REDACTED - contains credentials] ({} bytes)",
            xml_request.len()
        );

        let response = self
            .http
            .post(self.credentials.endpoint())
            .header("Content-Type", "text/xml")
            .body(xml_request.clone())
            .send()
            .await;

        // Zeroize the request XML as it contains credentials
        xml_request.zeroize();

        let response = response.context("Failed to send XML-RPC request")?;

        let status = response.status();

        // SECURITY: Enforce response size limit to prevent OOM from malicious servers
        let content_length = response.content_length().unwrap_or(0) as usize;
        if content_length > Self::MAX_RESPONSE_SIZE {
            return Err(anyhow::anyhow!(
                "Response too large ({} bytes, max {})",
                content_length,
                Self::MAX_RESPONSE_SIZE
            ));
        }

        let body = response
            .text()
            .await
            .context("Failed to read response body")?;

        if body.len() > Self::MAX_RESPONSE_SIZE {
            return Err(anyhow::anyhow!(
                "Response body too large ({} bytes, max {})",
                body.len(),
                Self::MAX_RESPONSE_SIZE
            ));
        }

        if !status.is_success() {
            // SECURITY: Don't log full response body as it may contain sensitive data
            tracing::error!("HTTP error: {} (response: {} bytes)", status, body.len());
            return Err(anyhow::anyhow!("HTTP request failed: {}", status));
        }

        // SECURITY: Only log response size, not content
        tracing::trace!("Response XML: {} bytes received", body.len());

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
        self.call("one.vm.info", vec![XmlRpcValue::Int(vm_id)])
            .await
    }

    /// Perform VM action (one.vm.action)
    pub async fn vm_action(&self, action: &str, vm_id: i32) -> Result<Value> {
        self.call(
            "one.vm.action",
            vec![
                XmlRpcValue::String(action.to_string()),
                XmlRpcValue::Int(vm_id),
            ],
        )
        .await
    }

    /// Migrate a VM to another host (one.vm.migrate)
    /// live: true for live migration, false for cold migration
    /// enforce: true to bypass capacity checks on target host
    /// ds_id: target datastore ID, -1 to use current
    pub async fn vm_migrate(
        &self,
        vm_id: i32,
        host_id: i32,
        live: bool,
        enforce: bool,
        ds_id: i32,
    ) -> Result<Value> {
        self.call(
            "one.vm.migrate",
            vec![
                XmlRpcValue::Int(vm_id),
                XmlRpcValue::Int(host_id),
                XmlRpcValue::Boolean(live),
                XmlRpcValue::Boolean(enforce),
                XmlRpcValue::Int(ds_id),
            ],
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
/// This function sanitizes error messages to prevent information disclosure
pub fn format_one_error(error: &anyhow::Error) -> String {
    let error_str = error.to_string();

    // Clean up common error patterns with safe messages
    if error_str.contains("401") || error_str.contains("Authentication") {
        return "Authentication failed. Check ONE_AUTH credentials.".to_string();
    }
    if error_str.contains("Connection refused") {
        return "Connection refused. Check ONE_XMLRPC endpoint.".to_string();
    }
    if error_str.contains("timeout") || error_str.contains("timed out") {
        return "Request timed out. Server may be unreachable.".to_string();
    }
    if error_str.contains("certificate") || error_str.contains("SSL") || error_str.contains("TLS") {
        return "TLS/SSL error. Check certificate configuration.".to_string();
    }

    // For OpenNebula API errors, extract just the message
    if let Some(start) = error_str.find("OpenNebula API error:") {
        let msg = &error_str[start..];
        // Truncate long error messages
        if msg.len() > 100 {
            return format!("{}...", &msg[..100]);
        }
        return msg.to_string();
    }

    // Generic fallback - don't expose internal details
    "An error occurred. Check logs for details.".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_string_contains, method};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Helper: build an XML-RPC success response wrapping an XML data string
    fn xmlrpc_success_response(xml_data: &str) -> String {
        // Escape XML inside the string value
        let escaped = xml_data
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        format!(
            r#"<?xml version="1.0"?>
<methodResponse><params><param><value><array><data>
  <value><boolean>1</boolean></value>
  <value><string>{}</string></value>
  <value><int>0</int></value>
</data></array></value></param></params></methodResponse>"#,
            escaped
        )
    }

    /// Helper: build an XML-RPC success response returning an int
    fn xmlrpc_success_int(val: i32) -> String {
        format!(
            r#"<?xml version="1.0"?>
<methodResponse><params><param><value><array><data>
  <value><boolean>1</boolean></value>
  <value><int>{}</int></value>
  <value><int>0</int></value>
</data></array></value></param></params></methodResponse>"#,
            val
        )
    }

    /// Helper: build an XML-RPC error response
    fn xmlrpc_error_response(msg: &str) -> String {
        let escaped = msg
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        format!(
            r#"<?xml version="1.0"?>
<methodResponse><params><param><value><array><data>
  <value><boolean>0</boolean></value>
  <value><string>{}</string></value>
  <value><int>-1</int></value>
</data></array></value></param></params></methodResponse>"#,
            escaped
        )
    }

    #[tokio::test]
    async fn test_list_hosts_returns_host_pool() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(body_string_contains("one.hostpool.info"))
            .respond_with(ResponseTemplate::new(200).set_body_string(xmlrpc_success_response(
                r#"<HOST_POOL><HOST><ID>0</ID><NAME>node-0</NAME><CLUSTER>default</CLUSTER><STATE>2</STATE><HOST_SHARE><RUNNING_VMS>5</RUNNING_VMS></HOST_SHARE></HOST><HOST><ID>1</ID><NAME>node-1</NAME><CLUSTER>default</CLUSTER><STATE>2</STATE><HOST_SHARE><RUNNING_VMS>3</RUNNING_VMS></HOST_SHARE></HOST></HOST_POOL>"#,
            )))
            .mount(&mock_server)
            .await;

        let client = OneClient::for_testing(&mock_server.uri());
        let result = client
            .list_hosts()
            .await
            .expect("list_hosts should succeed");

        let hosts = result
            .pointer("/HOST_POOL/HOST")
            .and_then(|v| v.as_array())
            .expect("Should have HOST array");
        assert_eq!(hosts.len(), 2);
        assert_eq!(hosts[0]["NAME"], "node-0");
        assert_eq!(hosts[1]["NAME"], "node-1");
        assert_eq!(hosts[1]["HOST_SHARE"]["RUNNING_VMS"], "3");
    }

    #[tokio::test]
    async fn test_vm_migrate_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(body_string_contains("one.vm.migrate"))
            .respond_with(ResponseTemplate::new(200).set_body_string(xmlrpc_success_int(42)))
            .mount(&mock_server)
            .await;

        let client = OneClient::for_testing(&mock_server.uri());
        let result = client
            .vm_migrate(42, 1, true, false, -1)
            .await
            .expect("vm_migrate should succeed");

        assert_eq!(result, serde_json::json!(42));
    }

    #[tokio::test]
    async fn test_vm_migrate_api_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(body_string_contains("one.vm.migrate"))
            .respond_with(
                ResponseTemplate::new(200).set_body_string(xmlrpc_error_response(
                    "[one.vm.migrate] Cannot migrate VM 42 to host 99: host not found",
                )),
            )
            .mount(&mock_server)
            .await;

        let client = OneClient::for_testing(&mock_server.uri());
        let result = client.vm_migrate(42, 99, true, false, -1).await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Cannot migrate VM 42"));
        assert!(err.contains("host not found"));
    }

    #[tokio::test]
    async fn test_vm_migrate_sends_correct_params() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(body_string_contains("one.vm.migrate"))
            // Check VM ID is in the request
            .and(body_string_contains("<int>42</int>"))
            // Check host ID is in the request
            .and(body_string_contains("<int>7</int>"))
            // Check live=true
            .and(body_string_contains("<boolean>1</boolean>"))
            .respond_with(ResponseTemplate::new(200).set_body_string(xmlrpc_success_int(42)))
            .mount(&mock_server)
            .await;

        let client = OneClient::for_testing(&mock_server.uri());
        let result = client.vm_migrate(42, 7, true, false, -1).await;

        assert!(result.is_ok(), "Should match all param matchers");
    }

    #[tokio::test]
    async fn test_list_hosts_single_host() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(body_string_contains("one.hostpool.info"))
            .respond_with(ResponseTemplate::new(200).set_body_string(xmlrpc_success_response(
                r#"<HOST_POOL><HOST><ID>0</ID><NAME>solo-node</NAME><CLUSTER>default</CLUSTER></HOST></HOST_POOL>"#,
            )))
            .mount(&mock_server)
            .await;

        let client = OneClient::for_testing(&mock_server.uri());
        let result = client.list_hosts().await.expect("Should succeed");

        // Single host: OpenNebula may return object instead of array
        let host_pool = result.pointer("/HOST_POOL/HOST").expect("Should have HOST");
        if let Some(arr) = host_pool.as_array() {
            assert_eq!(arr.len(), 1);
            assert_eq!(arr[0]["NAME"], "solo-node");
        } else {
            // Single object case
            assert_eq!(host_pool["NAME"], "solo-node");
        }
    }

    #[tokio::test]
    async fn test_get_vm_returns_vm_info() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(body_string_contains("one.vm.info"))
            .respond_with(ResponseTemplate::new(200).set_body_string(xmlrpc_success_response(
                r#"<VM><ID>100</ID><NAME>web-server</NAME><STATE>3</STATE><LCM_STATE>3</LCM_STATE></VM>"#,
            )))
            .mount(&mock_server)
            .await;

        let client = OneClient::for_testing(&mock_server.uri());
        let result = client.get_vm(100).await.expect("get_vm should succeed");
        assert_eq!(result["VM"]["ID"], "100");
        assert_eq!(result["VM"]["NAME"], "web-server");
    }

    #[tokio::test]
    async fn test_vm_action_resume() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(body_string_contains("one.vm.action"))
            .and(body_string_contains("resume"))
            .respond_with(ResponseTemplate::new(200).set_body_string(xmlrpc_success_int(100)))
            .mount(&mock_server)
            .await;

        let client = OneClient::for_testing(&mock_server.uri());
        let result = client
            .vm_action("resume", 100)
            .await
            .expect("vm_action should succeed");
        assert_eq!(result, serde_json::json!(100));
    }

    #[tokio::test]
    async fn test_list_vms_returns_pool() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(body_string_contains("one.vmpool.info"))
            .respond_with(ResponseTemplate::new(200).set_body_string(xmlrpc_success_response(
                r#"<VM_POOL><VM><ID>1</ID><NAME>vm1</NAME></VM><VM><ID>2</ID><NAME>vm2</NAME></VM></VM_POOL>"#,
            )))
            .mount(&mock_server)
            .await;

        let client = OneClient::for_testing(&mock_server.uri());
        let result = client
            .list_vms(-2, -1, -1, -1)
            .await
            .expect("list_vms should succeed");
        let vms = result
            .pointer("/VM_POOL/VM")
            .and_then(|v| v.as_array())
            .expect("Should have VM array");
        assert_eq!(vms.len(), 2);
    }

    #[tokio::test]
    async fn test_http_error_returns_err() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&mock_server)
            .await;

        let client = OneClient::for_testing(&mock_server.uri());
        let result = client.list_hosts().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("500"));
    }

    #[tokio::test]
    async fn test_full_migration_flow_simulation() {
        let mock_server = MockServer::start().await;

        // Step 1: list hosts
        Mock::given(method("POST"))
            .and(body_string_contains("one.hostpool.info"))
            .respond_with(ResponseTemplate::new(200).set_body_string(xmlrpc_success_response(
                r#"<HOST_POOL><HOST><ID>1</ID><NAME>target-host</NAME><CLUSTER>prod</CLUSTER><HOST_SHARE><RUNNING_VMS>2</RUNNING_VMS></HOST_SHARE></HOST><HOST><ID>2</ID><NAME>other-host</NAME><CLUSTER>prod</CLUSTER><HOST_SHARE><RUNNING_VMS>8</RUNNING_VMS></HOST_SHARE></HOST></HOST_POOL>"#,
            )))
            .mount(&mock_server)
            .await;

        // Step 2: migrate VM
        Mock::given(method("POST"))
            .and(body_string_contains("one.vm.migrate"))
            .respond_with(ResponseTemplate::new(200).set_body_string(xmlrpc_success_int(100)))
            .mount(&mock_server)
            .await;

        let client = OneClient::for_testing(&mock_server.uri());

        // Simulate: user lists hosts to pick a target
        let hosts_data = client.list_hosts().await.expect("list_hosts should work");
        let hosts = hosts_data
            .pointer("/HOST_POOL/HOST")
            .and_then(|v| v.as_array())
            .expect("Should have hosts");
        assert_eq!(hosts.len(), 2);

        // Simulate: user picks host ID 1 ("target-host")
        let target_host_id = hosts[0]["ID"].as_str().unwrap().parse::<i32>().unwrap();
        assert_eq!(target_host_id, 1);

        // Simulate: execute migration of VM 100 to host 1
        let migrate_result = client
            .vm_migrate(100, target_host_id, true, false, -1)
            .await
            .expect("migrate should succeed");
        assert_eq!(migrate_result, serde_json::json!(100));
    }
}

#[cfg(test)]
mod format_error_tests {
    use super::*;

    #[test]
    fn test_format_one_error_auth() {
        let err = anyhow::anyhow!("HTTP 401 Authentication failed");
        assert_eq!(
            format_one_error(&err),
            "Authentication failed. Check ONE_AUTH credentials."
        );
    }

    #[test]
    fn test_format_one_error_connection_refused() {
        let err = anyhow::anyhow!("Connection refused");
        assert_eq!(
            format_one_error(&err),
            "Connection refused. Check ONE_XMLRPC endpoint."
        );
    }

    #[test]
    fn test_format_one_error_timeout() {
        let err = anyhow::anyhow!("request timed out");
        assert_eq!(
            format_one_error(&err),
            "Request timed out. Server may be unreachable."
        );
    }

    #[test]
    fn test_format_one_error_tls() {
        let err = anyhow::anyhow!("SSL certificate problem");
        assert_eq!(
            format_one_error(&err),
            "TLS/SSL error. Check certificate configuration."
        );
    }

    #[test]
    fn test_format_one_error_api_error() {
        let err = anyhow::anyhow!("OpenNebula API error: VM not found");
        assert_eq!(format_one_error(&err), "OpenNebula API error: VM not found");
    }

    #[test]
    fn test_format_one_error_api_error_truncation() {
        let long_msg = format!("OpenNebula API error: {}", "x".repeat(200));
        let err = anyhow::anyhow!("{}", long_msg);
        let result = format_one_error(&err);
        assert!(result.len() <= 104); // 100 + "..."
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_format_one_error_generic() {
        let err = anyhow::anyhow!("some random internal error");
        assert_eq!(
            format_one_error(&err),
            "An error occurred. Check logs for details."
        );
    }
}
