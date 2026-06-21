//! Resource Fetcher
//!
//! Handles fetching resources from OpenNebula using the registry definitions.

use super::registry::{get_resource, ResourceFilter};
use super::sdk_dispatch::invoke_sdk_method;
use crate::one::OneClient;
use anyhow::Result;
use serde_json::Value;

/// Paginated result with next page token
pub struct PaginatedResult {
    pub items: Vec<Value>,
    pub next_token: Option<String>,
}

/// Fetch resources based on resource key
pub async fn fetch_resources(
    resource_key: &str,
    client: &OneClient,
    filters: &[ResourceFilter],
) -> Result<Vec<Value>> {
    let result = fetch_resources_paginated(resource_key, client, filters, None).await?;
    Ok(result.items)
}

/// Fetch resources with pagination support
pub async fn fetch_resources_paginated(
    resource_key: &str,
    client: &OneClient,
    filters: &[ResourceFilter],
    page_token: Option<&str>,
) -> Result<PaginatedResult> {
    let resource = get_resource(resource_key)
        .ok_or_else(|| anyhow::anyhow!("Unknown resource: {}", resource_key))?;

    // Build parameters from filters
    let mut params = resource.sdk_method_params.clone();
    if let Value::Object(ref mut map) = params {
        for filter in filters {
            map.insert(filter.name.clone(), Value::String(filter.values.join(",")));
        }
        if let Some(token) = page_token {
            map.insert("page_token".to_string(), Value::String(token.to_string()));
        }
    }

    // Call the SDK method
    let response =
        invoke_sdk_method(&resource.service, &resource.sdk_method, client, &params).await?;

    // Extract items from response using response_path
    let items = extract_items(&response, &resource.response_path)?;

    // OpenNebula doesn't have built-in pagination tokens, so we return None
    Ok(PaginatedResult {
        items,
        next_token: None,
    })
}

/// Extract items from response using a path like "VM_POOL.VM" or "HOST_POOL.HOST"
#[cfg(test)]
pub(crate) fn extract_items(response: &Value, path: &str) -> Result<Vec<Value>> {
    extract_items_impl(response, path)
}

#[cfg(not(test))]
fn extract_items(response: &Value, path: &str) -> Result<Vec<Value>> {
    extract_items_impl(response, path)
}

fn extract_items_impl(response: &Value, path: &str) -> Result<Vec<Value>> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = response;

    for part in parts {
        current = current
            .get(part)
            .ok_or_else(|| anyhow::anyhow!("Path '{}' not found in response", path))?;
    }

    match current {
        Value::Array(arr) => Ok(arr.clone()),
        Value::Object(_) => {
            // Single item - wrap in array
            Ok(vec![current.clone()])
        }
        Value::Null => Ok(Vec::new()),
        _ => Ok(Vec::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_items_array() {
        let response = json!({
            "VM_POOL": {
                "VM": [
                    {"ID": "1", "NAME": "vm1"},
                    {"ID": "2", "NAME": "vm2"}
                ]
            }
        });
        let items = extract_items(&response, "VM_POOL.VM").unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["ID"], "1");
    }

    #[test]
    fn test_extract_items_single_object() {
        let response = json!({
            "HOST_POOL": {
                "HOST": {"ID": "1", "NAME": "node-1"}
            }
        });
        let items = extract_items(&response, "HOST_POOL.HOST").unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["NAME"], "node-1");
    }

    #[test]
    fn test_extract_items_null() {
        let response = json!({
            "VM_POOL": {
                "VM": null
            }
        });
        let items = extract_items(&response, "VM_POOL.VM").unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn test_extract_items_missing_path() {
        let response = json!({"VM_POOL": {}});
        let result = extract_items(&response, "VM_POOL.VM");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
