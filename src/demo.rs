//! Demo mode - provides realistic fake data for testing the UI without OpenNebula

use serde_json::{json, Value};

/// Generate demo VM data
pub fn demo_vms() -> Vec<Value> {
    vec![
        json!({
            "ID": "0",
            "NAME": "web-frontend-01",
            "UNAME": "admin",
            "GNAME": "oneadmin",
            "STATE": "3",
            "LCM_STATE": "3",
            "HISTORY_RECORDS": {"HISTORY": {"HOSTNAME": "node-0"}},
            "TEMPLATE": {"CPU": "2", "MEMORY": "4096"}
        }),
        json!({
            "ID": "1",
            "NAME": "web-frontend-02",
            "UNAME": "admin",
            "GNAME": "oneadmin",
            "STATE": "3",
            "LCM_STATE": "3",
            "HISTORY_RECORDS": {"HISTORY": {"HOSTNAME": "node-1"}},
            "TEMPLATE": {"CPU": "2", "MEMORY": "4096"}
        }),
        json!({
            "ID": "2",
            "NAME": "api-server-01",
            "UNAME": "admin",
            "GNAME": "oneadmin",
            "STATE": "3",
            "LCM_STATE": "3",
            "HISTORY_RECORDS": {"HISTORY": {"HOSTNAME": "node-0"}},
            "TEMPLATE": {"CPU": "4", "MEMORY": "8192"}
        }),
        json!({
            "ID": "3",
            "NAME": "database-primary",
            "UNAME": "dbadmin",
            "GNAME": "production",
            "STATE": "3",
            "LCM_STATE": "3",
            "HISTORY_RECORDS": {"HISTORY": {"HOSTNAME": "node-2"}},
            "TEMPLATE": {"CPU": "8", "MEMORY": "32768"}
        }),
        json!({
            "ID": "4",
            "NAME": "database-replica",
            "UNAME": "dbadmin",
            "GNAME": "production",
            "STATE": "3",
            "LCM_STATE": "3",
            "HISTORY_RECORDS": {"HISTORY": {"HOSTNAME": "node-1"}},
            "TEMPLATE": {"CPU": "8", "MEMORY": "32768"}
        }),
        json!({
            "ID": "5",
            "NAME": "monitoring-stack",
            "UNAME": "ops",
            "GNAME": "infrastructure",
            "STATE": "3",
            "LCM_STATE": "3",
            "HISTORY_RECORDS": {"HISTORY": {"HOSTNAME": "node-2"}},
            "TEMPLATE": {"CPU": "2", "MEMORY": "4096"}
        }),
        json!({
            "ID": "6",
            "NAME": "ci-runner-01",
            "UNAME": "devops",
            "GNAME": "infrastructure",
            "STATE": "5",
            "LCM_STATE": "0",
            "HISTORY_RECORDS": {"HISTORY": {"HOSTNAME": "node-0"}},
            "TEMPLATE": {"CPU": "4", "MEMORY": "8192"}
        }),
        json!({
            "ID": "7",
            "NAME": "staging-app",
            "UNAME": "dev",
            "GNAME": "staging",
            "STATE": "8",
            "LCM_STATE": "0",
            "HISTORY_RECORDS": {"HISTORY": {"HOSTNAME": "node-1"}},
            "TEMPLATE": {"CPU": "2", "MEMORY": "2048"}
        }),
    ]
}

/// Generate demo VM template data
pub fn demo_templates() -> Vec<Value> {
    vec![
        json!({
            "ID": "0",
            "NAME": "tpl-small",
            "UNAME": "admin",
            "GNAME": "oneadmin",
            "TEMPLATE": {"CPU": "1", "MEMORY": "1024"}
        }),
        json!({
            "ID": "1",
            "NAME": "tpl-medium",
            "UNAME": "admin",
            "GNAME": "oneadmin",
            "TEMPLATE": {"CPU": "2", "MEMORY": "4096"}
        }),
        json!({
            "ID": "2",
            "NAME": "tpl-large",
            "UNAME": "admin",
            "GNAME": "oneadmin",
            "TEMPLATE": {"CPU": "4", "MEMORY": "8192"}
        }),
        json!({
            "ID": "3",
            "NAME": "tpl-gpu",
            "UNAME": "ops",
            "GNAME": "infrastructure",
            "TEMPLATE": {"CPU": "8", "MEMORY": "32768"}
        }),
    ]
}

/// Generate demo datastore data
pub fn demo_datastores() -> Vec<Value> {
    vec![
        json!({
            "ID": "0",
            "NAME": "default",
            "CLUSTER": "production",
            "TYPE": "0",
            "STATE": "0",
            "TOTAL_MB": "1048576",
            "FREE_MB": "524288",
            "IMAGES": {"ID": ["0", "1", "2"]}
        }),
        json!({
            "ID": "1",
            "NAME": "images",
            "CLUSTER": "production",
            "TYPE": "1",
            "STATE": "0",
            "TOTAL_MB": "2097152",
            "FREE_MB": "1572864",
            "IMAGES": {"ID": ["3", "4"]}
        }),
        json!({
            "ID": "2",
            "NAME": "system",
            "CLUSTER": "staging",
            "TYPE": "2",
            "STATE": "0",
            "TOTAL_MB": "524288",
            "FREE_MB": "393216",
            "IMAGES": {"ID": []}
        }),
    ]
}

/// Generate demo image data
pub fn demo_images() -> Vec<Value> {
    vec![
        json!({
            "ID": "0",
            "NAME": "Ubuntu 22.04",
            "UNAME": "admin",
            "GNAME": "oneadmin",
            "DATASTORE": "default",
            "TYPE": "0",
            "STATE": "1",
            "SIZE": "10240",
            "RUNNING_VMS": "3"
        }),
        json!({
            "ID": "1",
            "NAME": "CentOS 9 Stream",
            "UNAME": "admin",
            "GNAME": "oneadmin",
            "DATASTORE": "default",
            "TYPE": "0",
            "STATE": "1",
            "SIZE": "8192",
            "RUNNING_VMS": "2"
        }),
        json!({
            "ID": "2",
            "NAME": "Debian 12",
            "UNAME": "ops",
            "GNAME": "infrastructure",
            "DATASTORE": "default",
            "TYPE": "0",
            "STATE": "1",
            "SIZE": "8192",
            "RUNNING_VMS": "1"
        }),
        json!({
            "ID": "3",
            "NAME": "Alpine 3.19",
            "UNAME": "devops",
            "GNAME": "infrastructure",
            "DATASTORE": "images",
            "TYPE": "0",
            "STATE": "1",
            "SIZE": "512",
            "RUNNING_VMS": "1"
        }),
        json!({
            "ID": "4",
            "NAME": "Windows Server 2022",
            "UNAME": "admin",
            "GNAME": "oneadmin",
            "DATASTORE": "images",
            "TYPE": "0",
            "STATE": "1",
            "SIZE": "51200",
            "RUNNING_VMS": "0"
        }),
    ]
}

/// Generate demo virtual network data
pub fn demo_vnets() -> Vec<Value> {
    vec![
        json!({
            "ID": "0",
            "NAME": "public",
            "UNAME": "admin",
            "GNAME": "oneadmin",
            "CLUSTER": "production",
            "BRIDGE": "br0",
            "USED_LEASES": "6",
            "TEMPLATE": {"SIZE": "254"}
        }),
        json!({
            "ID": "1",
            "NAME": "private",
            "UNAME": "admin",
            "GNAME": "oneadmin",
            "CLUSTER": "production",
            "BRIDGE": "br1",
            "USED_LEASES": "8",
            "TEMPLATE": {"SIZE": "65534"}
        }),
        json!({
            "ID": "2",
            "NAME": "management",
            "UNAME": "ops",
            "GNAME": "infrastructure",
            "CLUSTER": "staging",
            "BRIDGE": "br-mgmt",
            "USED_LEASES": "4",
            "TEMPLATE": {"SIZE": "126"}
        }),
    ]
}

/// Generate demo cluster data
pub fn demo_clusters() -> Vec<Value> {
    vec![
        json!({
            "ID": "0",
            "NAME": "production",
            "HOSTS": {"ID": ["0", "1", "2"]},
            "VNETS": {"ID": ["0", "1"]},
            "DATASTORES": {"ID": ["0", "1"]}
        }),
        json!({
            "ID": "1",
            "NAME": "staging",
            "HOSTS": {"ID": ["3"]},
            "VNETS": {"ID": ["2"]},
            "DATASTORES": {"ID": ["2"]}
        }),
    ]
}

/// Generate demo user data
pub fn demo_users() -> Vec<Value> {
    vec![
        json!({
            "ID": "0",
            "NAME": "admin",
            "GNAME": "oneadmin",
            "AUTH_DRIVER": "core",
            "ENABLED": "1"
        }),
        json!({
            "ID": "1",
            "NAME": "dbadmin",
            "GNAME": "production",
            "AUTH_DRIVER": "core",
            "ENABLED": "1"
        }),
        json!({
            "ID": "2",
            "NAME": "ops",
            "GNAME": "infrastructure",
            "AUTH_DRIVER": "ldap",
            "ENABLED": "1"
        }),
        json!({
            "ID": "3",
            "NAME": "devops",
            "GNAME": "infrastructure",
            "AUTH_DRIVER": "ldap",
            "ENABLED": "1"
        }),
        json!({
            "ID": "4",
            "NAME": "dev",
            "GNAME": "staging",
            "AUTH_DRIVER": "ldap",
            "ENABLED": "0"
        }),
    ]
}

/// Generate demo group data
pub fn demo_groups() -> Vec<Value> {
    vec![
        json!({
            "ID": "0",
            "NAME": "oneadmin",
            "USERS": {"ID": ["0"]}
        }),
        json!({
            "ID": "1",
            "NAME": "production",
            "USERS": {"ID": ["1"]}
        }),
        json!({
            "ID": "2",
            "NAME": "infrastructure",
            "USERS": {"ID": ["2", "3"]}
        }),
        json!({
            "ID": "3",
            "NAME": "staging",
            "USERS": {"ID": ["4"]}
        }),
    ]
}

/// Generate demo zone data
pub fn demo_zones() -> Vec<Value> {
    vec![
        json!({
            "ID": "0",
            "NAME": "OpenNebula",
            "TEMPLATE": {"ENDPOINT": "http://demo.opennebula.local:2633/RPC2"}
        }),
    ]
}

/// Generate demo host data
pub fn demo_hosts() -> Vec<Value> {
    vec![
        json!({
            "ID": "0",
            "NAME": "node-0",
            "CLUSTER": "production",
            "STATE": "2",
            "HOST_SHARE": {
                "RUNNING_VMS": "3",
                "CPU_USAGE": "800",
                "MEM_USAGE": "20480"
            }
        }),
        json!({
            "ID": "1",
            "NAME": "node-1",
            "CLUSTER": "production",
            "STATE": "2",
            "HOST_SHARE": {
                "RUNNING_VMS": "3",
                "CPU_USAGE": "1200",
                "MEM_USAGE": "38912"
            }
        }),
        json!({
            "ID": "2",
            "NAME": "node-2",
            "CLUSTER": "production",
            "STATE": "2",
            "HOST_SHARE": {
                "RUNNING_VMS": "2",
                "CPU_USAGE": "1000",
                "MEM_USAGE": "36864"
            }
        }),
        json!({
            "ID": "3",
            "NAME": "node-3",
            "CLUSTER": "staging",
            "STATE": "2",
            "HOST_SHARE": {
                "RUNNING_VMS": "0",
                "CPU_USAGE": "0",
                "MEM_USAGE": "0"
            }
        }),
    ]
}
