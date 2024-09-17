use std::pin::Pin;

use log::error;
use serde::Deserialize;
use serde::Serialize;
use std::future::Future;

use crate::error::AgentError;
use crate::sh::run_command;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MachineInfo {
    #[serde(skip)]
    pub ip: String,

    #[serde(skip)]
    pub machine_type: String,

    pub hash: String,
    pub temp_sys: String,
    pub temp_hdd: String,
    pub cpu_occupy: String,
    pub cpu_model: String,

    #[serde(skip)]
    pub elapsed: String,

    pub sn: String,
    pub hdd_sn: String,
}

// impl json string to MachineInfo
impl From<&str> for MachineInfo {
    fn from(s: &str) -> Self {
        // if failed to parse, return default MachineInfo
        match serde_json::from_str(s) {
            Ok(info) => info,
            Err(e) => {
                error!("Failed to parse json: {}", e);
                MachineInfo::default()
            }
        }
    }
}

pub type AsyncOpType<T> = Pin<Box<dyn Future<Output = Result<T, AgentError>> + Send>>;

pub fn scan_ip_detail(ip: String, timeout_seconds: i64) -> AsyncOpType<MachineInfo> {
    Box::pin(async move {
        let cmd = "/opt/script/omni-collect.sh";

        let output = run_command(&ip, 22, "root", "dbos-miner", cmd)?;

        Ok(MachineInfo {
            ip,
            ..MachineInfo::from(output.as_str())
        })
    })
}

// test
#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;
    use log::info;
    use std::sync::Once;
    use tokio::runtime::Runtime;

    lazy_static! {
        static ref INIT: Once = Once::new();
    }

    fn init_logger() {
        INIT.call_once(|| {
            std::env::set_var("RUST_LOG", "info");
            env_logger::init();
        });
    }

    #[test]
    fn test_scan_ip_detail() {
        init_logger();

        let ip = "192.168.11.88".to_string();
        let timeout_seconds = 10;

        let rt = Runtime::new().unwrap();
        let result = rt.block_on(scan_ip_detail(ip, timeout_seconds));

        assert!(result.is_ok());
        let info = result.unwrap();
        info!("{:?}", info);
    }
}
