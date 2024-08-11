use color_eyre::Result;
use nix::sys::sysinfo::SysInfo;
use std::io;

pub fn get_current(info: &SysInfo) -> Result<String, io::Error> {
    let uptime_seconds = info.uptime().as_secs_f64();

    let total_minutes = (uptime_seconds / 60.0).round() as u64;
    let days = total_minutes / (60 * 24);
    let hours = (total_minutes % (60 * 24)) / 60;
    let minutes = total_minutes % 60;

    let mut parts = Vec::with_capacity(3);
    if days > 0 {
        parts.push(format!("{days} days"));
    }
    if hours > 0 || days > 0 {
        parts.push(format!("{hours} hours"));
    }
    if minutes > 0 || hours > 0 || days > 0 {
        parts.push(format!("{minutes} minutes"));
    }

    Ok(parts.join(", "))
}
