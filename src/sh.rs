// shell command wrapper

use std::process::Command;
use std::str;

use log::{error, info};

// use log::info;

use crate::error::AgentError;

// run scp command to cp spcified file to remote
pub fn run_scp(
    ip: &str,
    port: u16,
    user: &str,
    password: &str,
    local_file: &str,
    remote_file: &str,
    timeout_seconds: u64,
) -> Result<(), AgentError> {
    let output = Command::new("timeout")
        .arg(timeout_seconds.to_string())
        .arg("sshpass")
        .arg("-p")
        .arg(password)
        .arg("scp")
        .arg("-P")
        .arg(port.to_string())
        .arg("-o")
        .arg("StrictHostKeyChecking=no")
        .arg(local_file)
        .arg(format!("{}@{}:{}", user, ip, remote_file))
        .output()?;

    if output.status.success() {
        info!("scp success");
        return Ok(());
    } else {
        let stderr = str::from_utf8(&output.stderr)?;
        error!("scp error:{}", stderr);
        return Err(AgentError::CommandError(stderr.to_owned()));
    }
}

pub fn run_command(
    ip: &str,
    port: u16,
    user: &str,
    password: &str,
    command: &str,
    timeout_seconds: u64,
) -> Result<String, AgentError> {
    //info!("cmd: {}", command);
    let output = Command::new("timeout")
        .arg(timeout_seconds.to_string())
        .arg("sshpass")
        .arg("-p")
        .arg(password)
        .arg("ssh")
        .arg("-o")
        .arg("StrictHostKeyChecking=no")
        .arg("-p")
        .arg(port.to_string())
        .arg(format!("{}@{}", user, ip))
        .arg(command)
        .output()?;

    //info!("cmd status: {}", output.status);

    if output.status.success() {
        let stdout = str::from_utf8(&output.stdout)?;
        info!("stdout: {}", stdout);
        return Ok(stdout.to_owned());
    } else {
        let stderr = str::from_utf8(&output.stderr)?;
        //error!("cmd error:{}", stderr);
        return Err(AgentError::CommandError(stderr.to_owned()));
    }
}

// Test
#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;
    use std::sync::Once;

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
    fn test_run_command() {
        init_logger();

        let ip = "45.144.136.65";
        let port = 6002;
        let password = "ylkj..";
        let command = "ls";
        let user = "ylkj09";
        let result = run_command(ip, port, user, password, command, 5);
        //info!("{:?}", result);
        assert!(result.is_ok());
    }
}
