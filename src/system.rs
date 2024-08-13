use color_eyre::Result;
use nix::sys::statvfs::statvfs;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use crate::colors::{CYAN, GREEN, RED, RESET, YELLOW};
use std::env;

pub fn get_username_and_hostname() -> Result<String, io::Error> {
    let username = env::var("USER").unwrap_or_else(|_| "unknown_user".to_string());
    let hostname = nix::unistd::gethostname()?;
    let hostname = hostname.to_string_lossy();

    Ok(format!("{YELLOW}{username}{RED}@{GREEN}{hostname}"))
}

pub fn get_shell() -> Result<String, io::Error> {
    let shell_path = env::var("SHELL").unwrap_or_else(|_| "unknown_shell".to_string());
    let shell_name = shell_path.rsplit('/').next().unwrap_or("unknown_shell");

    Ok(shell_name.to_string())
}

pub fn get_root_disk_usage() -> Result<String, io::Error> {
    let vfs = statvfs("/")?;
    let block_size = vfs.block_size() as u64;
    let total_blocks = vfs.blocks();
    let available_blocks = vfs.blocks_available();

    let total_size = block_size * total_blocks;
    let used_size = total_size - (block_size * available_blocks);

    let total_size = total_size as f64 / (1024.0 * 1024.0 * 1024.0);
    let used_size = used_size as f64 / (1024.0 * 1024.0 * 1024.0);
    let usage = (used_size as f64 / total_size as f64) * 100.0;

    Ok(format!(
        "{used_size:.2} GiB / {total_size:.2} GiB ({CYAN}{usage:.0}%{RESET})"
    ))
}

pub fn get_memory_usage() -> Result<String, io::Error> {
    #[inline(always)]
    fn parse_memory_info() -> Result<(f64, f64), io::Error> {
        let path = Path::new("/proc/meminfo");
        let file = File::open(&path)?;
        let reader = io::BufReader::new(file);

        let mut meminfo = std::collections::HashMap::new();

        for line in reader.lines() {
            let line = line?;
            let mut parts = line.split_whitespace();
            if let (Some(key), Some(value), Some(_)) = (parts.next(), parts.next(), parts.next()) {
                let key = key.trim_end_matches(':');
                let value: u64 = value.parse().unwrap_or(0);
                meminfo.insert(key.to_string(), value);
            }
        }

        let total_memory = meminfo["MemTotal"];
        let used_memory = total_memory - meminfo["MemAvailable"];

        let used_memory_gb = used_memory as f64 / (1024.0 * 1024.0);
        let total_memory_gb = total_memory as f64 / (1024.0 * 1024.0);

        Ok((used_memory_gb, total_memory_gb))
    }

    let (used_memory, total_memory) = parse_memory_info()?;
    let percentage_used = (used_memory / total_memory * 100.0).round() as u64;

    Ok(format!(
        "{used_memory:.2} GiB / {total_memory:.2} GiB ({CYAN}{percentage_used}%{RESET})"
    ))
}
