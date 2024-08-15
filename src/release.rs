use color_eyre::Result;
use std::{
    fs::File,
    io::{self, Read},
};

pub fn get_system_info() -> nix::Result<String> {
    let utsname = nix::sys::utsname::uname()?;
    Ok(format!(
        "{} {} ({})",
        utsname.sysname().to_str().unwrap_or("Unknown"),
        utsname.release().to_str().unwrap_or("Unknown"),
        utsname.machine().to_str().unwrap_or("Unknown")
    ))
}

pub fn get_os_pretty_name() -> Result<String, io::Error> {
    let mut os_release_content = String::with_capacity(1024);
    File::open("/etc/os-release")?.read_to_string(&mut os_release_content)?;

    let pretty_name = os_release_content
        .lines()
        .find(|line| line.starts_with("PRETTY_NAME="))
        .map(|line| line.trim_start_matches("PRETTY_NAME=").trim_matches('"'));

    Ok(pretty_name.unwrap_or("Unknown").to_string())
}
