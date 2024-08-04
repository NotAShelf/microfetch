use color_eyre::Result;
use std::fs::{self, read_to_string};
use std::io;

// Try to detect OS type as accurately as possible and without depending on uname.
// /etc/os-release should generally imply Linux, and /etc/bsd-release would imply BSD system.
fn detect_os() -> Result<&'static str, io::Error> {
    if fs::metadata("/etc/os-release").is_ok() || fs::metadata("/usr/lib/os-release").is_ok() {
        Ok("Linux")
    } else if fs::metadata("/etc/rc.conf").is_ok() || fs::metadata("/etc/bsd-release").is_ok() {
        Ok("BSD")
    } else {
        Ok("Unknown")
    }
}

pub fn get_system_info() -> Result<String, io::Error> {
    let system = detect_os()?;

    let kernel_release = read_to_string("/proc/sys/kernel/osrelease")?;
    let kernel_release = kernel_release.trim();

    let architecture = read_to_string("/proc/sys/kernel/arch")?;
    let architecture = architecture.trim();

    let result = format!("{system} {kernel_release} ({architecture})");
    Ok(result)
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

    Ok(pretty_name.unwrap_or_else(|| "Unknown".to_string()))
}
