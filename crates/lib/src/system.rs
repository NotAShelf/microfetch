use core::mem::MaybeUninit;

#[cfg(target_os = "linux")]
use crate::syscall::read_file_fast;
use crate::{
  Error,
  StackWriter,
  UtsName,
  colors::Colors,
  syscall::{StatfsBuf, sys_statfs},
};

#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn write_username_and_hostname(
  w: &mut StackWriter,
  colors: &Colors,
  utsname: &UtsName,
) {
  let username = crate::getenv_str("USER").unwrap_or("unknown_user");
  let hostname = utsname.nodename();

  w.push_str(colors.yellow);
  w.push_str(username);
  w.push_str(colors.red);
  w.push_byte(b'@');
  w.push_str(colors.green);
  w.push_cstr(hostname);
  w.push_str(colors.reset);
}

#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn write_shell(w: &mut StackWriter) {
  let shell = crate::getenv_str("SHELL").unwrap_or("");
  if shell.is_empty() {
    w.push_str("unknown_shell");
  } else {
    let start = shell.rfind('/').map_or(0, |i| i + 1);
    w.push_str(&shell[start..]);
  }
}

/// Gets the root disk usage information.
///
/// # Errors
///
/// Returns an error if the filesystem information cannot be retrieved.
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn write_root_disk_usage(
  w: &mut StackWriter,
  colors: &Colors,
) -> Result<(), Error> {
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

  write_gib(w, used_bytes);
  w.push_str(" GiB / ");
  write_gib(w, total_bytes);
  w.push_str(" GiB (");
  w.push_str(colors.cyan);
  let pct = if total_bytes > 0 {
    used_bytes * 100 / total_bytes
  } else {
    0
  };
  w.push_u64(pct);
  w.push_byte(b'%');
  w.push_str(colors.reset);
  w.push_byte(b')');

  Ok(())
}

fn write_centi_gib(w: &mut StackWriter, centi_gib: u64) {
  w.push_u64(centi_gib / 100);
  w.push_byte(b'.');
  let frac = (centi_gib % 100) as u8;
  w.push_byte(b'0' + frac / 10);
  w.push_byte(b'0' + frac % 10);
}

fn write_gib(w: &mut StackWriter, bytes: u64) {
  write_centi_gib(w, (bytes * 100 + (1 << 29)) >> 30);
}

#[cfg(target_os = "linux")]
fn write_kb_as_gib(w: &mut StackWriter, kb: u64) {
  write_centi_gib(w, (kb * 100 + (1 << 19)) >> 20);
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

/// Writes the system memory usage information via `sysctl`/Mach (macOS).
///
/// # Errors
///
/// Returns an error if the memory statistics cannot be retrieved.
#[cfg(target_os = "macos")]
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn write_memory_usage(
  w: &mut StackWriter,
  colors: &Colors,
) -> Result<(), Error> {
  let (used_bytes, total_bytes) =
    crate::syscall::macos_meminfo().ok_or(Error::OsError(0))?;
  let percentage_used = if total_bytes > 0 {
    (used_bytes * 100 + total_bytes / 2) / total_bytes
  } else {
    0
  };

  write_gib(w, used_bytes);
  w.push_str(" GiB / ");
  write_gib(w, total_bytes);
  w.push_str(" GiB (");
  w.push_str(colors.cyan);
  w.push_u64(percentage_used);
  w.push_byte(b'%');
  w.push_str(colors.reset);
  w.push_byte(b')');

  Ok(())
}

/// Gets the system memory usage information.
///
/// # Errors
///
/// Returns an error if `/proc/meminfo` cannot be read.
#[cfg(target_os = "linux")]
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn write_memory_usage(
  w: &mut StackWriter,
  colors: &Colors,
) -> Result<(), Error> {
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

  write_kb_as_gib(w, used_kb);
  w.push_str(" GiB / ");
  write_kb_as_gib(w, total_kb);
  w.push_str(" GiB (");
  w.push_str(colors.cyan);
  w.push_u64(percentage_used);
  w.push_byte(b'%');
  w.push_str(colors.reset);
  w.push_byte(b')');

  Ok(())
}
