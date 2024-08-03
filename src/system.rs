use libc::statfs as libc_statfs_struct;
use std::env;
use std::ffi::CString;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process::Command;

use crate::colors::{CYAN, GREEN, RED, RESET, YELLOW};

pub fn get_username_and_hostname() -> Result<String, io::Error> {
    let username = env::var("USER").unwrap_or_else(|_| "unknown_user".to_string());
    let output = Command::new("hostname").output()?;
    let hostname = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok(format!("{YELLOW}{}{RED}@{GREEN}{}", username, hostname))
}

pub fn get_disk_usage() -> Result<String, io::Error> {
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

pub fn get_memory_usage() -> Result<String, io::Error> {
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
