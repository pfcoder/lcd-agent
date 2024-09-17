use std::pin::Pin;

use log::{error, info};
use serde::Deserialize;
use serde::Serialize;
use std::future::Future;

use crate::error::AgentError;
use crate::sh::run_command;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MachineInfo {
    #[serde(skip_deserializing)]
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

pub fn scan_ip_detail(ip: String, timeout_seconds: u64) -> AsyncOpType<MachineInfo> {
    Box::pin(async move {
        let cmd = "/opt/script/omni-collect.sh";

        let output = run_command(&ip, 22, "root", "dbos-miner", cmd, timeout_seconds)?;

        Ok(MachineInfo {
            ip,
            ..MachineInfo::from(output.as_str())
        })
    })
}

pub async fn batch_scan(
    ip: &str,
    runtime_handle: &tokio::runtime::Handle,
) -> Result<Vec<MachineInfo>, AgentError> {
    info!("scan ip: {}", ip);

    let ip_prefix = ip.split('.').take(3).collect::<Vec<&str>>().join(".");
    let mut handles = vec![];
    for i in 1..256 {
        let ip = format!("{}.{}", ip_prefix, i);
        handles.push(runtime_handle.spawn(async move { scan_ip_detail(ip, 5).await }));
    }

    let result = futures::future::join_all(handles).await;

    let mut machines = vec![];
    for res in result {
        match res {
            Ok(Ok(machine)) => {
                machines.push(machine);
            }
            Ok(Err(_e)) => {
                // info!("scan_and_update_db error: {:?}", e);
            }
            Err(_e) => {
                // info!("scan_and_update_db join error: {:?}", e);
            }
        }
    }

    info!("scan ip done: {}", machines.len());

    Ok(machines)
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

    #[test]
    fn test_batch_scan() {
        init_logger();

        let ip = "192.168.11.1";
        // let rt = Runtime::new().unwrap();

        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(256)
            .enable_all()
            .build()
            .unwrap();

        let result = rt.block_on(batch_scan(ip, rt.handle()));

        assert!(result.is_ok());
        let machines = result.unwrap();
        info!("{:?}", machines);
    }
}
