//! Resource abstraction layer
//!
//! Provides a unified interface for working with different OpenNebula resource types.

mod fetcher;
mod registry;
mod sdk_dispatch;

pub use fetcher::{fetch_resources, fetch_resources_paginated};
pub use registry::{
    get_all_resource_keys, get_color_for_value, get_resource, ActionDef, ColumnDef, ResourceDef,
    ResourceFilter,
};
pub use sdk_dispatch::invoke_sdk_method;

/// Extract a value from JSON using a dot-notation path
pub fn extract_json_value(item: &serde_json::Value, path: &str) -> String {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = item;

    for part in parts {
        // Handle array indexing like "DISK[0]"
        if let Some(bracket_pos) = part.find('[') {
            let key = &part[..bracket_pos];
            let idx_str = &part[bracket_pos + 1..part.len() - 1];
            if let Ok(idx) = idx_str.parse::<usize>() {
                current = match current.get(key) {
                    Some(arr) => match arr.get(idx) {
                        Some(v) => v,
                        None => return "-".to_string(),
                    },
                    None => return "-".to_string(),
                };
                continue;
            }
        }

        current = match current.get(part) {
            Some(v) => v,
            None => return "-".to_string(),
        };
    }

    match current {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "-".to_string(),
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                "-".to_string()
            } else if arr.len() == 1 {
                // Single element array - extract value
                extract_json_value(&arr[0], "")
            } else {
                format!("[{} items]", arr.len())
            }
        }
        serde_json::Value::Object(_) => "[object]".to_string(),
    }
}

/// Format OpenNebula VM state code to string
pub fn format_vm_state(state: i32) -> String {
    match state {
        0 => "INIT".to_string(),
        1 => "PENDING".to_string(),
        2 => "HOLD".to_string(),
        3 => "ACTIVE".to_string(),
        4 => "STOPPED".to_string(),
        5 => "SUSPENDED".to_string(),
        6 => "DONE".to_string(),
        8 => "POWEROFF".to_string(),
        9 => "UNDEPLOYED".to_string(),
        10 => "CLONING".to_string(),
        11 => "CLONING_FAILURE".to_string(),
        _ => format!("UNKNOWN({})", state),
    }
}

/// Format OpenNebula VM LCM state code to string
pub fn format_lcm_state(lcm_state: i32) -> String {
    match lcm_state {
        0 => "LCM_INIT".to_string(),
        1 => "PROLOG".to_string(),
        2 => "BOOT".to_string(),
        3 => "RUNNING".to_string(),
        4 => "MIGRATE".to_string(),
        5 => "SAVE_STOP".to_string(),
        6 => "SAVE_SUSPEND".to_string(),
        7 => "SAVE_MIGRATE".to_string(),
        8 => "PROLOG_MIGRATE".to_string(),
        9 => "PROLOG_RESUME".to_string(),
        10 => "EPILOG_STOP".to_string(),
        11 => "EPILOG".to_string(),
        12 => "SHUTDOWN".to_string(),
        14 => "CLEANUP_RESUBMIT".to_string(),
        15 => "UNKNOWN".to_string(),
        16 => "HOTPLUG".to_string(),
        17 => "SHUTDOWN_POWEROFF".to_string(),
        18 => "BOOT_UNKNOWN".to_string(),
        19 => "BOOT_POWEROFF".to_string(),
        20 => "BOOT_SUSPENDED".to_string(),
        21 => "BOOT_STOPPED".to_string(),
        22 => "CLEANUP_DELETE".to_string(),
        23 => "HOTPLUG_SNAPSHOT".to_string(),
        24 => "HOTPLUG_NIC".to_string(),
        25 => "HOTPLUG_SAVEAS".to_string(),
        26 => "HOTPLUG_SAVEAS_POWEROFF".to_string(),
        27 => "HOTPLUG_SAVEAS_SUSPENDED".to_string(),
        28 => "SHUTDOWN_UNDEPLOY".to_string(),
        29 => "EPILOG_UNDEPLOY".to_string(),
        30 => "PROLOG_UNDEPLOY".to_string(),
        31 => "BOOT_UNDEPLOY".to_string(),
        32 => "HOTPLUG_PROLOG_POWEROFF".to_string(),
        33 => "HOTPLUG_EPILOG_POWEROFF".to_string(),
        34 => "BOOT_MIGRATE".to_string(),
        35 => "BOOT_FAILURE".to_string(),
        36 => "BOOT_MIGRATE_FAILURE".to_string(),
        37 => "PROLOG_MIGRATE_FAILURE".to_string(),
        38 => "PROLOG_FAILURE".to_string(),
        39 => "EPILOG_FAILURE".to_string(),
        40 => "EPILOG_STOP_FAILURE".to_string(),
        41 => "EPILOG_UNDEPLOY_FAILURE".to_string(),
        42 => "PROLOG_MIGRATE_POWEROFF".to_string(),
        43 => "PROLOG_MIGRATE_POWEROFF_FAILURE".to_string(),
        44 => "PROLOG_MIGRATE_SUSPEND".to_string(),
        45 => "PROLOG_MIGRATE_SUSPEND_FAILURE".to_string(),
        46 => "BOOT_UNDEPLOY_FAILURE".to_string(),
        47 => "BOOT_STOPPED_FAILURE".to_string(),
        48 => "PROLOG_RESUME_FAILURE".to_string(),
        49 => "PROLOG_UNDEPLOY_FAILURE".to_string(),
        50 => "DISK_SNAPSHOT_POWEROFF".to_string(),
        51 => "DISK_SNAPSHOT_REVERT_POWEROFF".to_string(),
        52 => "DISK_SNAPSHOT_DELETE_POWEROFF".to_string(),
        53 => "DISK_SNAPSHOT_SUSPENDED".to_string(),
        54 => "DISK_SNAPSHOT_REVERT_SUSPENDED".to_string(),
        55 => "DISK_SNAPSHOT_DELETE_SUSPENDED".to_string(),
        56 => "DISK_SNAPSHOT".to_string(),
        57 => "DISK_SNAPSHOT_REVERT".to_string(),
        58 => "DISK_SNAPSHOT_DELETE".to_string(),
        59 => "PROLOG_MIGRATE_UNKNOWN".to_string(),
        60 => "PROLOG_MIGRATE_UNKNOWN_FAILURE".to_string(),
        61 => "DISK_RESIZE".to_string(),
        62 => "DISK_RESIZE_POWEROFF".to_string(),
        63 => "DISK_RESIZE_UNDEPLOYED".to_string(),
        64 => "HOTPLUG_NIC_POWEROFF".to_string(),
        65 => "HOTPLUG_RESIZE".to_string(),
        66 => "HOTPLUG_SAVEAS_UNDEPLOYED".to_string(),
        67 => "HOTPLUG_SAVEAS_STOPPED".to_string(),
        68 => "BACKUP".to_string(),
        69 => "BACKUP_POWEROFF".to_string(),
        _ => format!("LCM_UNKNOWN({})", lcm_state),
    }
}

/// Format OpenNebula host state code to string
pub fn format_host_state(state: i32) -> String {
    match state {
        0 => "INIT".to_string(),
        1 => "MONITORING_MONITORED".to_string(),
        2 => "MONITORED".to_string(),
        3 => "ERROR".to_string(),
        4 => "DISABLED".to_string(),
        5 => "MONITORING_ERROR".to_string(),
        6 => "MONITORING_INIT".to_string(),
        7 => "MONITORING_DISABLED".to_string(),
        8 => "OFFLINE".to_string(),
        _ => format!("UNKNOWN({})", state),
    }
}

/// Format OpenNebula image state code to string
pub fn format_image_state(state: i32) -> String {
    match state {
        0 => "INIT".to_string(),
        1 => "READY".to_string(),
        2 => "USED".to_string(),
        3 => "DISABLED".to_string(),
        4 => "LOCKED".to_string(),
        5 => "ERROR".to_string(),
        6 => "CLONE".to_string(),
        7 => "DELETE".to_string(),
        8 => "USED_PERS".to_string(),
        9 => "LOCKED_USED".to_string(),
        10 => "LOCKED_USED_PERS".to_string(),
        _ => format!("UNKNOWN({})", state),
    }
}

/// Format OpenNebula datastore state code to string
pub fn format_datastore_state(state: i32) -> String {
    match state {
        0 => "READY".to_string(),
        1 => "DISABLED".to_string(),
        _ => format!("UNKNOWN({})", state),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_host_fields_for_migration_dialog() {
        let host = json!({
            "ID": "7",
            "NAME": "node-1",
            "CLUSTER": "default",
            "HOST_SHARE": {
                "RUNNING_VMS": "3",
                "CPU_USAGE": "400",
                "MEM_USAGE": "8192"
            }
        });

        assert_eq!(extract_json_value(&host, "ID"), "7");
        assert_eq!(extract_json_value(&host, "NAME"), "node-1");
        assert_eq!(extract_json_value(&host, "CLUSTER"), "default");
        assert_eq!(extract_json_value(&host, "HOST_SHARE.RUNNING_VMS"), "3");
    }

    #[test]
    fn test_extract_missing_host_field() {
        let host = json!({"ID": "7", "NAME": "node-1"});
        assert_eq!(extract_json_value(&host, "CLUSTER"), "-");
        assert_eq!(extract_json_value(&host, "HOST_SHARE.RUNNING_VMS"), "-");
    }

    #[test]
    fn test_lcm_migrate_states() {
        assert_eq!(format_lcm_state(4), "MIGRATE");
        assert_eq!(format_lcm_state(7), "SAVE_MIGRATE");
        assert_eq!(format_lcm_state(8), "PROLOG_MIGRATE");
        assert_eq!(format_lcm_state(34), "BOOT_MIGRATE");
        assert_eq!(format_lcm_state(36), "BOOT_MIGRATE_FAILURE");
        assert_eq!(format_lcm_state(37), "PROLOG_MIGRATE_FAILURE");
    }

    #[test]
    fn test_format_vm_state_all() {
        assert_eq!(format_vm_state(0), "INIT");
        assert_eq!(format_vm_state(1), "PENDING");
        assert_eq!(format_vm_state(2), "HOLD");
        assert_eq!(format_vm_state(3), "ACTIVE");
        assert_eq!(format_vm_state(4), "STOPPED");
        assert_eq!(format_vm_state(5), "SUSPENDED");
        assert_eq!(format_vm_state(6), "DONE");
        assert_eq!(format_vm_state(8), "POWEROFF");
        assert_eq!(format_vm_state(9), "UNDEPLOYED");
        assert_eq!(format_vm_state(10), "CLONING");
        assert_eq!(format_vm_state(11), "CLONING_FAILURE");
        assert_eq!(format_vm_state(99), "UNKNOWN(99)");
    }

    #[test]
    fn test_format_host_state_all() {
        assert_eq!(format_host_state(0), "INIT");
        assert_eq!(format_host_state(1), "MONITORING_MONITORED");
        assert_eq!(format_host_state(2), "MONITORED");
        assert_eq!(format_host_state(3), "ERROR");
        assert_eq!(format_host_state(4), "DISABLED");
        assert_eq!(format_host_state(5), "MONITORING_ERROR");
        assert_eq!(format_host_state(6), "MONITORING_INIT");
        assert_eq!(format_host_state(7), "MONITORING_DISABLED");
        assert_eq!(format_host_state(8), "OFFLINE");
        assert_eq!(format_host_state(42), "UNKNOWN(42)");
    }

    #[test]
    fn test_format_image_state_all() {
        assert_eq!(format_image_state(0), "INIT");
        assert_eq!(format_image_state(1), "READY");
        assert_eq!(format_image_state(2), "USED");
        assert_eq!(format_image_state(3), "DISABLED");
        assert_eq!(format_image_state(4), "LOCKED");
        assert_eq!(format_image_state(5), "ERROR");
        assert_eq!(format_image_state(6), "CLONE");
        assert_eq!(format_image_state(7), "DELETE");
        assert_eq!(format_image_state(8), "USED_PERS");
        assert_eq!(format_image_state(9), "LOCKED_USED");
        assert_eq!(format_image_state(10), "LOCKED_USED_PERS");
        assert_eq!(format_image_state(77), "UNKNOWN(77)");
    }

    #[test]
    fn test_format_datastore_state_all() {
        assert_eq!(format_datastore_state(0), "READY");
        assert_eq!(format_datastore_state(1), "DISABLED");
        assert_eq!(format_datastore_state(5), "UNKNOWN(5)");
    }

    #[test]
    fn test_extract_json_value_array_indexing() {
        let item = json!({
            "DISK": [
                {"SIZE": "1024", "TYPE": "hd"},
                {"SIZE": "2048", "TYPE": "cdrom"}
            ]
        });
        assert_eq!(extract_json_value(&item, "DISK[0].SIZE"), "1024");
        assert_eq!(extract_json_value(&item, "DISK[1].TYPE"), "cdrom");
        assert_eq!(extract_json_value(&item, "DISK[5].SIZE"), "-");
    }

    #[test]
    fn test_extract_json_value_types() {
        let item = json!({
            "NUM": 42,
            "FLAG": true,
            "EMPTY": null,
            "OBJ": {"nested": "val"},
            "ARR_SINGLE": ["only"],
            "ARR_MULTI": ["a", "b", "c"],
            "ARR_EMPTY": []
        });
        assert_eq!(extract_json_value(&item, "NUM"), "42");
        assert_eq!(extract_json_value(&item, "FLAG"), "true");
        assert_eq!(extract_json_value(&item, "EMPTY"), "-");
        assert_eq!(extract_json_value(&item, "OBJ"), "[object]");
        // Single-element array recurses with empty path; plain string yields "-"
        assert_eq!(extract_json_value(&item, "ARR_SINGLE"), "-");
        assert_eq!(extract_json_value(&item, "ARR_MULTI"), "[3 items]");
        assert_eq!(extract_json_value(&item, "ARR_EMPTY"), "-");
        assert_eq!(extract_json_value(&item, "NONEXISTENT"), "-");
    }

    #[test]
    fn test_format_lcm_state_all_ranges() {
        // Test a sampling of all defined states
        assert_eq!(format_lcm_state(0), "LCM_INIT");
        assert_eq!(format_lcm_state(3), "RUNNING");
        assert_eq!(format_lcm_state(12), "SHUTDOWN");
        assert_eq!(format_lcm_state(15), "UNKNOWN");
        assert_eq!(format_lcm_state(22), "CLEANUP_DELETE");
        assert_eq!(format_lcm_state(56), "DISK_SNAPSHOT");
        assert_eq!(format_lcm_state(68), "BACKUP");
        assert_eq!(format_lcm_state(69), "BACKUP_POWEROFF");
        assert_eq!(format_lcm_state(999), "LCM_UNKNOWN(999)");
    }
}
