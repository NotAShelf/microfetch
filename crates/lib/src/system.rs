include!(concat!(env!("OUT_DIR"), "/color_helpers.rs"));

use alloc::string::String;
use core::mem::MaybeUninit;

use crate::{
  Error,
  UtsName,
  colors::Colors,
  syscall::{StatfsBuf, read_file_fast, sys_statfs},
};

#[must_use]
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn get_username_and_hostname(utsname: &UtsName) -> String {
  let username = crate::getenv_str("USER").unwrap_or("unknown_user");
  let hostname = utsname.nodename().to_str().unwrap_or("unknown_host");

  // Get colors (checking NO_COLOR only once)
  let no_color = crate::colors::is_no_color();
  let colors = Colors::new(no_color);

  let capacity = colors.yellow.len()
    + username.len()
    + colors.red.len()
    + 1
    + colors.green.len()
    + hostname.len()
    + colors.reset.len();
  let mut result = String::with_capacity(capacity);

  result.push_str(colors.yellow);
  result.push_str(username);
  result.push_str(colors.red);
  result.push('@');
  result.push_str(colors.green);
  result.push_str(hostname);
  result.push_str(colors.reset);

  result
}

#[must_use]
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn get_shell() -> String {
  let shell = crate::getenv_str("SHELL").unwrap_or("");
  let start = shell.rfind('/').map_or(0, |i| i + 1);
  if shell.is_empty() {
    String::from("unknown_shell")
  } else {
    String::from(&shell[start..])
  }
}

/// Gets the root disk usage information.
///
/// # Errors
///
/// Returns an error if the filesystem information cannot be retrieved.
#[cfg_attr(feature = "hotpath", hotpath::measure)]
#[allow(clippy::cast_precision_loss)]
pub fn get_root_disk_usage() -> Result<String, Error> {
  let mut vfs = MaybeUninit::<StatfsBuf>::uninit();
  let path = b"/\0";

  if unsafe { sys_statfs(path.as_ptr(), vfs.as_mut_ptr()) } != 0 {
    return Err(Error::last_os_error());
  }

  let vfs = unsafe { vfs.assume_init() };
  #[allow(clippy::cast_sign_loss)]
  let block_size = vfs.f_bsize as u64;
  let total_blocks = vfs.f_blocks;
  let available_blocks = vfs.f_bavail;

  let total_size = block_size * total_blocks;
  let used_size = total_size - (block_size * available_blocks);

  let total_size = total_size as f64 / (1024.0 * 1024.0 * 1024.0);
  let used_size = used_size as f64 / (1024.0 * 1024.0 * 1024.0);
  let usage = (used_size / total_size) * 100.0;

  let no_color = crate::colors::is_no_color();
  let colors = Colors::new(no_color);

  let mut result = String::with_capacity(64);

  // Manual float formatting
  write_float(&mut result, used_size, 2);
  result.push_str(" GiB / ");
  write_float(&mut result, total_size, 2);
  result.push_str(" GiB (");
  result.push_str(colors.icon);
  write_float(&mut result, usage, 0);
  result.push('%');
  result.push_str(colors.reset);
  result.push_str(RPAREN);

  Ok(result)
}

/// Write a float to string with specified decimal places
#[allow(
  clippy::cast_sign_loss,
  clippy::cast_possible_truncation,
  clippy::cast_precision_loss
)]
fn write_float(s: &mut String, val: f64, decimals: u32) {
  // Handle integer part
  let int_part = val as u64;
  write_u64(s, int_part);

  if decimals > 0 {
    s.push('.');

    // Calculate fractional part
    let mut frac = val - int_part as f64;
    for _ in 0..decimals {
      frac *= 10.0;
      let digit = frac as u8;
      s.push((b'0' + digit) as char);
      frac -= f64::from(digit);
    }
  }
}

/// Round an f64 to nearest integer (`f64::round` is not in core)
#[allow(
  clippy::cast_precision_loss,
  clippy::cast_possible_truncation,
  clippy::cast_sign_loss
)]
fn round_f64(x: f64) -> f64 {
  if x >= 0.0 {
    let int_part = x as u64 as f64;
    let frac = x - int_part;
    if frac >= 0.5 {
      int_part + 1.0
    } else {
      int_part
    }
  } else {
    let int_part = (-x) as u64 as f64;
    let frac = -x - int_part;
    if frac >= 0.5 {
      -(int_part + 1.0)
    } else {
      -int_part
    }
  }
}

/// Write a u64 to string
pub fn write_u64(s: &mut String, mut n: u64) {
  if n == 0 {
    s.push('0');
    return;
  }

  let mut buf = [0u8; 20];
  let mut i = 20;

  while n > 0 {
    i -= 1;
    buf[i] = b'0' + (n % 10) as u8;
    n /= 10;
  }

  // SAFETY: buf contains only ASCII digits
  s.push_str(unsafe { core::str::from_utf8_unchecked(&buf[i..]) });
}

/// Fast integer parsing without stdlib overhead
#[inline]
fn parse_u64_fast(s: &[u8]) -> u64 {
  let mut result = 0u64;
  for &byte in s {
    if byte.is_ascii_digit() {
      result = result * 10 + u64::from(byte - b'0');
    } else {
      break;
    }
  }
  result
}

/// Gets the system memory usage information.
///
/// # Errors
///
/// Returns an error if `/proc/meminfo` cannot be read.
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn get_memory_usage() -> Result<String, Error> {
  #[cfg_attr(feature = "hotpath", hotpath::measure)]
  fn parse_memory_info() -> Result<(f64, f64), Error> {
    let mut total_memory_kb = 0u64;
    let mut available_memory_kb = 0u64;
    let mut buffer = [0u8; 1024];

    // Use fast syscall-based file reading
    let bytes_read = read_file_fast("/proc/meminfo", &mut buffer)
      .map_err(Error::from_raw_os_error)?;
    let meminfo = &buffer[..bytes_read];

    // Fast scanning for MemTotal and MemAvailable
    let mut offset = 0;
    let mut found_total = false;
    let mut found_available = false;

    while offset < meminfo.len() && (!found_total || !found_available) {
      let remaining = &meminfo[offset..];

      // Find newline or end
      let line_end = remaining
        .iter()
        .position(|&b| b == b'\n')
        .unwrap_or(remaining.len());
      let line = &remaining[..line_end];

      if line.starts_with(b"MemTotal:") {
        // Skip "MemTotal:" and whitespace
        let mut pos = 9;
        while pos < line.len() && line[pos].is_ascii_whitespace() {
          pos += 1;
        }
        total_memory_kb = parse_u64_fast(&line[pos..]);
        found_total = true;
      } else if line.starts_with(b"MemAvailable:") {
        // Skip "MemAvailable:" and whitespace
        let mut pos = 13;
        while pos < line.len() && line[pos].is_ascii_whitespace() {
          pos += 1;
        }
        available_memory_kb = parse_u64_fast(&line[pos..]);
        found_available = true;
      }

      offset += line_end + 1;
    }

    #[allow(clippy::cast_precision_loss)]
    let total_gb = total_memory_kb as f64 / 1024.0 / 1024.0;
    #[allow(clippy::cast_precision_loss)]
    let available_gb = available_memory_kb as f64 / 1024.0 / 1024.0;
    let used_memory_gb = total_gb - available_gb;

    Ok((used_memory_gb, total_gb))
  }

  let (used_memory, total_memory) = parse_memory_info()?;
  #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
  let percentage_used = round_f64(used_memory / total_memory * 100.0) as u64;

  let no_color = crate::colors::is_no_color();
  let colors = Colors::new(no_color);

  let mut result = String::with_capacity(64);

  write_float(&mut result, used_memory, 2);
  result.push_str(" GiB / ");
  write_float(&mut result, total_memory, 2);
  result.push_str(" GiB (");
  result.push_str(colors.icon);
  write_u64(&mut result, percentage_used);
  result.push('%');
  result.push_str(colors.reset);
  result.push_str(RPAREN);

  Ok(result)
}
