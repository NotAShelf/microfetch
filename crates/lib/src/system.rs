use core::mem::MaybeUninit;

#[cfg(target_os = "linux")]
use crate::syscall::read_file_fast;
use crate::{
  StackWriter,
  UtsName,
  colors::Colors,
  syscall::{StatfsBuf, sys_statfs},
};

#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn write_username_and_hostname(
  writer: &mut StackWriter,
  colors: &Colors,
  utsname: &UtsName,
) {
  let username = crate::getenv_str("USER").unwrap_or("unknown_user");
  let hostname = utsname.nodename();

  writer.push_str(colors.yellow);
  writer.push_str(username);
  writer.push_str(colors.red);
  writer.push_byte(b'@');
  writer.push_str(colors.green);
  writer.push_cstr(hostname);
  writer.push_str(colors.reset);
}

#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn write_shell(writer: &mut StackWriter) {
  let shell = crate::getenv_str("SHELL").unwrap_or("");
  if shell.is_empty() {
    writer.push_str("unknown_shell");
  } else {
    let start = shell.rfind('/').map_or(0, |pos| pos + 1);
    writer.push_str(&shell[start..]);
  }
}

/// Writes the root disk usage, or nothing if the filesystem information cannot
/// be retrieved.
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn write_root_disk_usage(writer: &mut StackWriter, colors: &Colors) {
  let mut vfs = MaybeUninit::<StatfsBuf>::uninit();
  let path = b"/\0";

  if unsafe { sys_statfs(path.as_ptr(), vfs.as_mut_ptr()) } != 0 {
    return;
  }

  let vfs = unsafe { vfs.assume_init() };
  #[allow(
    clippy::unnecessary_fallible_conversions,
    reason = "f_bsize is a signed long on most targets but u32 on macOS, \
              s390x, and 32-bit"
  )]
  let Some(block_size) = u64::try_from(vfs.f_bsize).ok() else {
    return;
  };
  let total_blocks = vfs.f_blocks;
  let available_blocks = vfs.f_bavail;

  let total_bytes = block_size * total_blocks;
  let used_bytes = total_bytes - (block_size * available_blocks);

  write_gib(writer, used_bytes);
  writer.push_str(" GiB / ");
  write_gib(writer, total_bytes);
  writer.push_str(" GiB (");
  writer.push_str(colors.cyan);
  let pct = (used_bytes * 100).checked_div(total_bytes).unwrap_or(0);
  writer.push_u64(pct);
  writer.push_byte(b'%');
  writer.push_str(colors.reset);
  writer.push_byte(b')');
}

fn write_centi_gib(writer: &mut StackWriter, centi_gib: u64) {
  writer.push_u64(centi_gib / 100);
  writer.push_byte(b'.');
  let frac = (centi_gib % 100) as u8;
  writer.push_byte(b'0' + frac / 10);
  writer.push_byte(b'0' + frac % 10);
}

fn write_gib(writer: &mut StackWriter, bytes: u64) {
  write_centi_gib(writer, (bytes * 100 + (1 << 29)) >> 30);
}

#[cfg(target_os = "linux")]
fn write_kb_as_gib(writer: &mut StackWriter, kb: u64) {
  write_centi_gib(writer, (kb * 100 + (1 << 19)) >> 20);
}

/// Fast integer parsing without stdlib overhead
#[cfg(target_os = "linux")]
#[inline]
fn parse_u64_fast(bytes: &[u8]) -> u64 {
  let mut result = 0u64;
  for &byte in bytes {
    if byte.is_ascii_digit() {
      result = result * 10 + u64::from(byte - b'0');
    } else {
      break;
    }
  }
  result
}

/// Writes the system memory usage information via `sysctl`/Mach (macOS).
/// Writes nothing if the memory statistics cannot be retrieved.
#[cfg(target_os = "macos")]
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn write_memory_usage(writer: &mut StackWriter, colors: &Colors) {
  let Some((used_bytes, total_bytes)) = crate::syscall::macos_meminfo() else {
    return;
  };
  let percentage_used = (used_bytes * 100 + total_bytes / 2)
    .checked_div(total_bytes)
    .unwrap_or(0);

  write_gib(writer, used_bytes);
  writer.push_str(" GiB / ");
  write_gib(writer, total_bytes);
  writer.push_str(" GiB (");
  writer.push_str(colors.cyan);
  writer.push_u64(percentage_used);
  writer.push_byte(b'%');
  writer.push_str(colors.reset);
  writer.push_byte(b')');
}

/// Gets the system memory usage information.
/// Writes nothing if `/proc/meminfo` cannot be read.
#[cfg(target_os = "linux")]
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn write_memory_usage(writer: &mut StackWriter, colors: &Colors) {
  #[cfg_attr(feature = "hotpath", hotpath::measure)]
  fn parse_memory_info() -> Option<(u64, u64)> {
    let mut total_memory_kb = 0u64;
    let mut available_memory_kb = 0u64;
    let mut buffer = [0u8; 1024];

    // Use fast syscall-based file reading
    let bytes_read = read_file_fast("/proc/meminfo", &mut buffer).ok()?;
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
        .position(|&byte| byte == b'\n')
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

    Some((total_memory_kb - available_memory_kb, total_memory_kb))
  }

  let Some((used_kb, total_kb)) = parse_memory_info() else {
    return;
  };
  let percentage_used = (used_kb * 100 + total_kb / 2)
    .checked_div(total_kb)
    .unwrap_or(0);

  write_kb_as_gib(writer, used_kb);
  writer.push_str(" GiB / ");
  write_kb_as_gib(writer, total_kb);
  writer.push_str(" GiB (");
  writer.push_str(colors.cyan);
  writer.push_u64(percentage_used);
  writer.push_byte(b'%');
  writer.push_str(colors.reset);
  writer.push_byte(b')');
}
