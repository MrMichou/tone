//! Resource abstraction layer
//!
//! Provides a unified interface for working with different OpenNebula resource types.

mod fetcher;
mod registry;
mod sdk_dispatch;

pub use fetcher::{fetch_resources, fetch_resources_paginated, PaginatedResult};
pub use registry::{
    get_all_resource_keys, get_color_for_value, get_color_map, get_resource, ActionDef,
    ColumnDef, ConfirmConfig, ResourceDef, ResourceFilter, SubResourceDef,
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

/// Format bytes to human readable string
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
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
