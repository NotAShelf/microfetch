use color_eyre::Result;
use std::fs::read_to_string;
use std::io;

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
    let os_release_content = read_to_string("/etc/os-release")?;
    let pretty_name = os_release_content
        .lines()
        .find(|line| line.starts_with("PRETTY_NAME="))
        .map(|line| {
            line.trim_start_matches("PRETTY_NAME=")
                .trim_matches('"')
                .to_string()
        });

    Ok(pretty_name.unwrap_or("Unknown".to_string()))
}
