use alloc::string::String;
use core::mem::MaybeUninit;

#[cfg(target_os = "linux")]
use crate::syscall::read_file_fast;
use crate::{
  Error,
  UtsName,
  colors::Colors,
  syscall::{StatfsBuf, sys_statfs},
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

  let total_bytes = block_size * total_blocks;
  let used_bytes = total_bytes - (block_size * available_blocks);

  let no_color = crate::colors::is_no_color();
  let colors = Colors::new(no_color);

  let mut result = String::with_capacity(64);

  write_gib(&mut result, used_bytes);
  result.push_str(" GiB / ");
  write_gib(&mut result, total_bytes);
  result.push_str(" GiB (");
  result.push_str(colors.cyan);
  let pct = if total_bytes > 0 {
    used_bytes * 100 / total_bytes
  } else {
    0
  };
  write_u64(&mut result, pct);
  result.push('%');
  result.push_str(colors.reset);
  result.push(')');

  Ok(result)
}

fn write_centi_gib(s: &mut String, centi_gib: u64) {
  write_u64(s, centi_gib / 100);
  s.push('.');
  let frac = (centi_gib % 100) as u8;
  s.push((b'0' + frac / 10) as char);
  s.push((b'0' + frac % 10) as char);
}

fn write_gib(s: &mut String, bytes: u64) {
  write_centi_gib(s, (bytes * 100 + (1 << 29)) >> 30);
}

#[cfg(target_os = "linux")]
fn write_kb_as_gib(s: &mut String, kb: u64) {
  write_centi_gib(s, (kb * 100 + (1 << 19)) >> 20);
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
#[cfg(target_os = "linux")]
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

/// Gets the system memory usage information via `sysctl`/Mach (macOS).
///
/// # Errors
///
/// Returns an error if the memory statistics cannot be retrieved.
#[cfg(target_os = "macos")]
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn get_memory_usage() -> Result<String, Error> {
  let (used_bytes, total_bytes) =
    crate::syscall::macos_meminfo().ok_or(Error::OsError(0))?;
  let percentage_used = if total_bytes > 0 {
    (used_bytes * 100 + total_bytes / 2) / total_bytes
  } else {
    0
  };
  let colors = Colors::new(crate::colors::is_no_color());
  let mut result = String::with_capacity(64);

  write_gib(&mut result, used_bytes);
  result.push_str(" GiB / ");
  write_gib(&mut result, total_bytes);
  result.push_str(" GiB (");
  result.push_str(colors.cyan);
  write_u64(&mut result, percentage_used);
  result.push('%');
  result.push_str(colors.reset);
  result.push(')');

  Ok(result)
}

/// Gets the system memory usage information.
///
/// # Errors
///
/// Returns an error if `/proc/meminfo` cannot be read.
#[cfg(target_os = "linux")]
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn get_memory_usage() -> Result<String, Error> {
  #[cfg_attr(feature = "hotpath", hotpath::measure)]
  fn parse_memory_info() -> Result<(u64, u64), Error> {
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

    Ok((total_memory_kb - available_memory_kb, total_memory_kb))
  }

  let (used_kb, total_kb) = parse_memory_info()?;
  let percentage_used = if total_kb > 0 {
    (used_kb * 100 + total_kb / 2) / total_kb
  } else {
    0
  };

  let no_color = crate::colors::is_no_color();
  let colors = Colors::new(no_color);

  let mut result = String::with_capacity(64);

  write_kb_as_gib(&mut result, used_kb);
  result.push_str(" GiB / ");
  write_kb_as_gib(&mut result, total_kb);
  result.push_str(" GiB (");
  result.push_str(colors.cyan);
  write_u64(&mut result, percentage_used);
  result.push('%');
  result.push_str(colors.reset);
  result.push(')');

  Ok(result)
}
