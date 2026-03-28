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
