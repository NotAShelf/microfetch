use color_eyre::Result;
use std::fs::{self, read_to_string};
use std::io::{self, Read};

// Try to detect OS type as accurately as possible and without depending on uname.
// /etc/os-release should generally imply Linux, and /etc/bsd-release would imply BSD system.
fn detect_os() -> Result<String, io::Error> {
    if fs::metadata("/etc/os-release").is_ok() || fs::metadata("/usr/lib/os-release").is_ok() {
        Ok("Linux".to_string())
    } else if fs::metadata("/etc/rc.conf").is_ok() || fs::metadata("/etc/bsd-release").is_ok() {
        Ok("BSD".to_string())
    } else {
        Ok("Unknown".to_string())
    }
}

fn get_architecture() -> Result<String, io::Error> {
    // Read architecture from /proc/sys/kernel/arch
    let mut arch = String::new();
    fs::File::open("/proc/sys/kernel/arch")?.read_to_string(&mut arch)?;
    let arch = arch.trim().to_string();
    Ok(arch)
}

pub fn get_system_info() -> Result<String, io::Error> {
    let system = detect_os()?;

    let mut kernel_release = String::new();
    fs::File::open("/proc/sys/kernel/osrelease")?.read_to_string(&mut kernel_release)?;
    let kernel_release = kernel_release.trim().to_string();

    let mut cpuinfo = String::new();
    fs::File::open("/proc/cpuinfo")?.read_to_string(&mut cpuinfo)?;

    let architecture = get_architecture()?;

    let result = format!("{system} {kernel_release} ({architecture})");
    Ok(result)
}

pub fn get_os_pretty_name() -> Option<String> {
    let os_release_content = read_to_string("/etc/os-release").ok()?;
    let os_release_lines: Vec<&str> = os_release_content.lines().collect();
    let pretty_name = os_release_lines
        .iter()
        .find(|line| line.starts_with("PRETTY_NAME="))
        .map(|line| {
            line.trim_start_matches("PRETTY_NAME=")
                .trim_matches('"')
                .to_string()
        });

    pretty_name
}
