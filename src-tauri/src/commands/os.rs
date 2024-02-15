use super::Error;
use std::fs;
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

pub fn create_file(file_path: &str) -> Result<(), Error> {
    match Command::new("touch").args([&file_path]).output() {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                let why = format!("Failed to create file: {stdout}\n{stderr}");
                Err(Error::IO(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    why,
                )))
            }
        }
        Err(e) => {
            let why = format!("Failed to create file: {e:?}");
            Err(Error::IO(std::io::Error::new(
                std::io::ErrorKind::Other,
                why,
            )))
        }
    }
}

pub fn delete_file(file_path: &str) -> Result<(), Error> {
    match Command::new("rm").args([&file_path]).output() {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                let why = format!("Failed to remove file: {stdout}\n{stderr}");
                Err(Error::IO(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    why,
                )))
            }
        }
        Err(e) => {
            let why = format!("Failed to remove file: {e:?}");
            Err(Error::IO(std::io::Error::new(
                std::io::ErrorKind::Other,
                why,
            )))
        }
    }
}

#[must_use]
pub fn directory_exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}

pub fn is_directory(path: &str) -> Result<bool, Error> {
    Ok(fs::metadata(path)?.is_dir())
}

pub fn create_directory(path: &str) -> Result<(), Error> {
    fs::create_dir(path)?;
    Ok(())
}
