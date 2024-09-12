// shell command wrapper

use std::process::Command;
use std::str;

use crate::error::AgentError;

pub fn run_command(ip: &str, password: &str, command: &str) -> Result<String, AgentError> {
    let output = Command::new("sshpass")
        .arg("-p")
        .arg(password)
        .arg("ssh")
        .arg("-o")
        .arg("StrictHostKeyChecking=no")
        .arg(format!("root@{}", ip))
        .arg(command)
        .output()?;

    if output.status.success() {
        let stdout = str::from_utf8(&output.stdout)?;
        return Ok(stdout.to_owned());
    } else {
        let stderr = str::from_utf8(&output.stderr)?;
        return Err(AgentError::CommandError(stderr.to_owned()));
    }
}
