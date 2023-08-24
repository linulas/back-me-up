use super::Error;
use std::process::Command;

pub fn get_hostname() -> Result<String, Error> {
    // TODO: use 'hostname' command for windows
    let uname = Command::new("uname")
        .arg("-n") // -n flag to get the network node hostname
        .output()?;

    if uname.status.success() {
        Ok(String::from_utf8_lossy(&uname.stdout).trim().to_string())
    } else {
        let stdout = String::from_utf8_lossy(&uname.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&uname.stderr).trim().to_string();
        let why = format!("Getting hostname with command 'uname' failed: {stdout}\n{stderr}");
        Err(Error::IO(std::io::Error::new(
            std::io::ErrorKind::Other,
            why,
        )))
    }
}
