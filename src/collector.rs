use std::pin::Pin;

use log::{error, info};
use serde::Deserialize;
use serde::Serialize;
use std::future::Future;

use crate::error::AgentError;
use crate::sh::{run_command, run_scp};

/*
{
    "index": "0",
    "name": "NVIDIA GeForce RTX 3070",
    "power": "172.27",
    "temperature": "89"
  }, */
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub index: String,
    pub name: String,
    pub power: String,
    pub temperature: String,
}

/*
{
    "timestamp": "2024-10-13T10:22:57",
    "gpu_index": "0",
    "one_min": "445925",
    "five_min": "445325",
    "fifteen_min": "332917",
    "thirty_min": "306742",
    "sixty_min": "309270"
  } */
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ProverInfo {
    pub timestamp: String,
    pub gpu_index: String,
    pub one_min: String,
    pub five_min: String,
    pub fifteen_min: String,
    pub thirty_min: String,
    pub sixty_min: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MachineInfo {
    #[serde(skip_deserializing)]
    pub ip: String,

    pub gpu_info: Vec<GpuInfo>,
    pub prover_info: Vec<ProverInfo>,
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

pub fn deploy_to_ip(
    ip: &str,
    pwd: &str,
    ver: &str,
    addr: &str,
    timeout_seconds: u64,
) -> AsyncOpType<()> {
    let ip = ip.to_string();
    let pwd = pwd.to_string();
    let ver = ver.to_string();
    let addr = addr.to_string();

    Box::pin(async move {
        let _output = run_scp(
            &ip,
            22,
            "root",
            &pwd,
            "./machine.tgz",
            "/opt/machine.tgz",
            timeout_seconds,
        )?;

        // scp success, then perform remote tar -xvzf
        let cmd = "tar -xvzf /opt/machine.tgz -C /opt/";
        let _output = run_command(&ip, 22, "root", &pwd, cmd, timeout_seconds)?;
        // perform remote shell script /opt/omni-gpu-agent/zk-ins.sh
        // let cmd = "/opt/omni-gpu-agent/zk-ins.sh";
        let cmd = format!("/opt/res/machine/zk-ins.sh {} {}", ver, addr);
        let _output = run_command(&ip, 22, "root", &pwd, &cmd, timeout_seconds)?;

        // this will be a long time depends on target machine network

        Ok(())
    })
}

pub fn reboot_ip(ip: &str, pwd: &str, timeout_seconds: u64) -> AsyncOpType<()> {
    let ip = ip.to_string();
    let pwd = pwd.to_string();
    Box::pin(async move {
        let cmd = "reboot";
        let _output = run_command(&ip, 22, "root", &pwd, cmd, timeout_seconds)?;

        Ok(())
    })
}

pub fn scan_ip_detail(ip: &str, pwd: &str, timeout_seconds: u64) -> AsyncOpType<(MachineInfo)> {
    let ip = ip.to_string();
    let pwd = pwd.to_string();
    Box::pin(async move {
        let cmd = "/opt/omni-gpu-agent/collect.sh";

        match run_command(&ip, 22, "root", &pwd, cmd, timeout_seconds) {
            Ok(output) => Ok(MachineInfo {
                ip: ip.to_string(),
                ..MachineInfo::from(output.as_str())
            }),
            Err(_e) => Ok(MachineInfo {
                ip: ip.to_string(),
                ..MachineInfo::default()
            }),
        }
    })
}

pub fn reboot_prover(ip: &str, pwd: &str, timeout_seconds: u64) -> AsyncOpType<()> {
    let ip = ip.to_string();
    let pwd = pwd.to_string();
    Box::pin(async move {
        let cmd = "systemctl restart aleo.service";
        let _output = run_command(&ip, 22, "root", &pwd, cmd, timeout_seconds)?;

        Ok(())
    })
}

pub async fn batch_scan(
    ip: &str,
    pwd: &str,
    runtime_handle: &tokio::runtime::Handle,
) -> Result<Vec<MachineInfo>, AgentError> {
    info!("scan ip: {}", ip);

    let ip_prefix = ip.split('.').take(3).collect::<Vec<&str>>().join(".");
    let mut handles = vec![];
    for i in 1..256 {
        let ip = format!("{}.{}", ip_prefix, i);
        let pwd = pwd.to_string();
        handles.push(runtime_handle.spawn(async move { scan_ip_detail(&ip, &pwd, 5).await }));
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

pub async fn batch_deploy(
    ip: &str,
    pwd: &str,
    ver: &str,
    addr: &str,
    runtime_handle: &tokio::runtime::Handle,
) -> Result<(), AgentError> {
    let ip_prefix = ip.split('.').take(3).collect::<Vec<&str>>().join(".");
    let mut handles = vec![];
    for i in 1..256 {
        let ip = format!("{}.{}", ip_prefix, i);
        let pwd = pwd.to_string();
        let ver = ver.to_string();
        let addr = addr.to_string();
        handles.push(
            runtime_handle.spawn(async move { deploy_to_ip(&ip, &pwd, &ver, &addr, 5).await }),
        );
    }

    let _result = futures::future::join_all(handles).await;

    Ok(())
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

        let ip = "192.168.187.73";
        let timeout_seconds = 10;

        let rt = Runtime::new().unwrap();
        let result = rt.block_on(scan_ip_detail(ip, "123456.", timeout_seconds));
        info!("result: {:?}", result);

        assert!(result.is_ok());
        let info = result.unwrap();
        info!("{:?}", info);
    }

    #[test]
    fn test_deploy_to_ip() {
        init_logger();

        let ip = "192.168.187.90";
        let timeout_seconds = 10;

        let rt = Runtime::new().unwrap();
        let result = rt.block_on(deploy_to_ip(
            ip,
            "123456.",
            "0.2.3",
            "aleo1spkkxewxj2dl2lgdps9xr28093p5nxsvjv55g2unmqfu0hmwyuysmf4qp3",
            timeout_seconds,
        ));
        info!("result: {:?}", result);

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

        let result = rt.block_on(batch_scan(ip, "123456.", rt.handle()));

        assert!(result.is_ok());
        let machines = result.unwrap();
        info!("{:?}", machines);
    }
}
