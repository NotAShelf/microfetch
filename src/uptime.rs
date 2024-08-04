use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

pub fn get_current() -> Result<String, io::Error> {
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
