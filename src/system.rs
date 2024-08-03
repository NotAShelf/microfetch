use nix::sys::statvfs::statvfs;

use std::env;
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

pub fn get_root_disk_usage() -> Result<String, Box<dyn std::error::Error>> {
    let vfs = statvfs("/")?;
    let block_size = vfs.block_size() as u64;
    let total_blocks = vfs.blocks();
    let available_blocks = vfs.blocks_available();

    let total_size = block_size * total_blocks;
    let used_size = total_size - (block_size * available_blocks);

    let total_size_gib = total_size as f64 / (1024.0 * 1024.0 * 1024.0);
    let used_size_gib = used_size as f64 / (1024.0 * 1024.0 * 1024.0);
    let usage_percentage = (used_size as f64 / total_size as f64) * 100.0;

    Ok(format!(
        "{:.2} GiB / {:.2} GiB ({CYAN}{:.0}%{RESET})",
        used_size_gib, total_size_gib, usage_percentage
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
