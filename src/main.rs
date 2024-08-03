use color_eyre::{Report, Result};
use libc::statfs as libc_statfs_struct;
use std::env;
use std::ffi::CString;
use std::fs::{self, File};
use std::io::{self, BufRead, Read};
use std::path::Path;
use std::process::Command;

const RESET: &str = "\x1b[0m";
const BLUE: &str = "\x1b[34m";
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";

fn main() -> Result<(), Report> {
    color_eyre::install()?;

    let user_info = get_username_and_hostname().expect("Failed to get username and hostname");
    let os_name = get_os_pretty_name().expect("Failed to get OS name");
    let kernel_version = get_system_info().expect("Failed to get kernel info");
    let uptime = get_system_uptime().expect("Failed to get uptime");
    let window_manager = get_desktop_info().expect("Failed to get desktop info");
    let memory_usage = get_memory_usage().expect("Failed to get memory usage");
    let storage = get_disk_usage().expect("Failed to get storage info");

    // Construct the ASCII art with dynamic OS name

    println!(
        "
{CYAN}  ▗▄   {BLUE}▗▄ ▄▖         {} ~{RESET}
{CYAN} ▄▄🬸█▄▄▄{BLUE}🬸█▛ {CYAN}▃        {CYAN}  {BLUE}System{RESET}    {}
{BLUE}   ▟▛    ▜{CYAN}▃▟🬕        {CYAN}  {BLUE}Kernel{RESET}    {}
{BLUE}🬋🬋🬫█      {CYAN}█🬛🬋🬋       {CYAN}  {BLUE}Uptime{RESET}    {}
{BLUE} 🬷▛🮃{CYAN}▙    ▟▛          {CYAN}  {BLUE}WM{RESET}        {}
{BLUE} 🮃 {CYAN}▟█🬴{BLUE}▀▀▀█🬴▀▀        {CYAN}󰍛  {BLUE}Memory{RESET}    {}
{CYAN}  ▝▀ ▀▘   {BLUE}▀▘         {CYAN}󱥎  {BLUE}Storage{RESET}   {}
    ",
        user_info, os_name, kernel_version, uptime, window_manager, memory_usage, storage
    );

    Ok(())
}

fn get_username_and_hostname() -> Result<String, io::Error> {
    let username = env::var("USER").unwrap_or_else(|_| "unknown_user".to_string());
    let output = Command::new("hostname").output()?;
    let hostname = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok(format!("{YELLOW}{}{RED}@{GREEN}{}", username, hostname))
}

fn get_disk_usage() -> Result<String, io::Error> {
    let path = CString::new("/").expect("CString::new failed");

    let mut fs_stat: libc_statfs_struct = unsafe { std::mem::zeroed() };
    let result = unsafe { libc::statfs(path.as_ptr(), &mut fs_stat) };

    if result != 0 {
        return Err(io::Error::last_os_error());
    }

    let block_size = fs_stat.f_bsize as u64;
    let total_blocks = fs_stat.f_blocks as u64;
    let free_blocks = fs_stat.f_bfree as u64;

    let total_size_bytes = total_blocks * block_size;
    let used_size_bytes = (total_blocks - free_blocks) * block_size;

    let total_size_gib = total_size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    let used_size_gib = used_size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    let percentage_used = (used_size_bytes as f64 / total_size_bytes as f64) * 100.0;

    let formatted_total_size = format!("{:.2}", total_size_gib);
    let formatted_used_size = format!("{:.2}", used_size_gib);
    let formatted_percentage_used = format!("{:.0}", percentage_used);

    Ok(format!(
        "{} GiB / {} GiB ({CYAN}{}%{RESET})",
        formatted_used_size, formatted_total_size, formatted_percentage_used
    ))
}

fn get_memory_usage() -> Result<String, io::Error> {
    fn parse_memory_info() -> Result<(f64, f64), io::Error> {
        let path = Path::new("/proc/meminfo");
        let file = File::open(path)?;
        let reader = io::BufReader::new(file);

        let mut total_memory_kb = 0.0;
        let mut available_memory_kb = 0.0;

        for line in reader.lines() {
            let line = line?;
            if line.starts_with("MemTotal:") {
                total_memory_kb = line
                    .split_whitespace()
                    .nth(1)
                    .ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidData, "Failed to parse MemTotal")
                    })?
                    .parse::<f64>()
                    .unwrap_or(0.0);
            } else if line.starts_with("MemAvailable:") {
                available_memory_kb = line
                    .split_whitespace()
                    .nth(1)
                    .ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidData, "Failed to parse MemAvailable")
                    })?
                    .parse::<f64>()
                    .unwrap_or(0.0);
            }
        }

        let total_memory_gb = total_memory_kb / (1024.0 * 1024.0);
        let available_memory_gb = available_memory_kb / (1024.0 * 1024.0);
        let used_memory_gb = total_memory_gb - available_memory_gb;

        Ok((used_memory_gb, total_memory_gb))
    }

    let (used_memory, total_memory) = parse_memory_info()?;
    let percentage_used = (used_memory / total_memory * 100.0).round() as u64;

    Ok(format!(
        "{:.2} GiB / {:.2} GiB ({CYAN}{}%{RESET})",
        used_memory, total_memory, percentage_used
    ))
}

fn get_desktop_info() -> Result<String, io::Error> {
    let desktop_env = env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
    let display_backend = env::var("XDG_SESSION_TYPE").unwrap_or_default();

    // Trim "none+" from the start of desktop_env if present
    // XXX: This is a workaround for NixOS modules that set XDG_CURRENT_DESKTOP to "none+foo"
    // instead of just "foo"
    let desktop_env = desktop_env.trim_start_matches("none+");

    Ok(format!("{} ({})", desktop_env, display_backend))
}

fn get_os_pretty_name() -> Option<String> {
    let os_release_content = fs::read_to_string("/etc/os-release").ok()?;
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

fn get_system_uptime() -> Result<String, io::Error> {
    let path = Path::new("/proc/uptime");
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);

    let line = reader
        .lines()
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to read uptime"))??;

    let uptime_seconds: f64 = line
        .split_whitespace()
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to parse uptime"))?
        .parse()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    // calculate days, hours, and minutes
    let total_minutes = (uptime_seconds / 60.0).round() as u64;
    let days = total_minutes / (60 * 24);
    let hours = (total_minutes % (60 * 24)) / 60;
    let minutes = total_minutes % 60;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{} days", days));
    }

    if hours > 0 || days > 0 {
        parts.push(format!("{} hours", hours));
    }

    if minutes > 0 || hours > 0 || days > 0 {
        parts.push(format!("{} minutes", minutes));
    }

    Ok(parts.join(", "))
}

fn get_system_info() -> Result<String, io::Error> {
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

    let system = detect_os()?;

    let mut kernel_release = String::new();
    fs::File::open("/proc/sys/kernel/osrelease")?.read_to_string(&mut kernel_release)?;
    let kernel_release = kernel_release.trim().to_string(); // Remove any trailing newline

    let mut cpuinfo = String::new();
    fs::File::open("/proc/cpuinfo")?.read_to_string(&mut cpuinfo)?;

    let architecture = if let Some(line) = cpuinfo.lines().find(|line| line.contains("flags")) {
        if line.contains("lm") {
            "x86_64".to_string()
        } else {
            "unknown".to_string()
        }
    } else {
        "unknown".to_string()
    };

    let result = format!("{} {} ({})", system, kernel_release, architecture);
    Ok(result)
}
