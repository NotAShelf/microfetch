use color_eyre::Result;
use nix::sys::statvfs::statvfs;
use nix::sys::sysinfo::SysInfo;

use std::env;
use std::io::{self};

use crate::colors::{CYAN, GREEN, RED, RESET, YELLOW};

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

pub fn get_memory_usage(info: &SysInfo) -> String {
    #[inline(always)]
    fn parse_memory_info(info: &SysInfo) -> (f64, f64) {
        let total_memory_kb = (info.ram_total() / 1024) as f64;
        let available_memory_kb = (info.ram_unused() / 1024) as f64;

        let total_memory_gb = total_memory_kb / (1024.0 * 1024.0);
        let available_memory_gb = available_memory_kb / (1024.0 * 1024.0);
        let used_memory_gb = total_memory_gb - available_memory_gb;

        (used_memory_gb, total_memory_gb)
    }

    let (used_memory, total_memory) = parse_memory_info(info);
    let percentage_used = (used_memory / total_memory * 100.0).round() as u64;

    format!("{used_memory:.2} GiB / {total_memory:.2} GiB ({CYAN}{percentage_used}%{RESET})")
}
