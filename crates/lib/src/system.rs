use core::mem::MaybeUninit;

#[cfg(target_os = "linux")]
use microfetch_asm::read_file_fast;
use microfetch_asm::{StatfsBuf, sys_statfs};

use crate::{StackWriter, UtsName, colors::Colors};

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
    writer.push_str(shell.rsplit_once('/').map_or(shell, |(_, name)| name));
  }
}

/// Writes the root disk usage, or nothing if the filesystem information cannot
/// be retrieved.
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn write_root_disk_usage(writer: &mut StackWriter, colors: &Colors) {
  let mut vfs_buf = MaybeUninit::<StatfsBuf>::uninit();
  let path = b"/\0";

  if unsafe { sys_statfs(path.as_ptr(), vfs_buf.as_mut_ptr()) } != 0_i32 {
    return;
  }

  let vfs = unsafe { vfs_buf.assume_init() };
  let block_size = vfs.block_size();

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
  let frac = u8::try_from(centi_gib % 100).unwrap_or(0);
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

/// Fast integer parsing without stdlib overhead.
#[cfg(target_os = "linux")]
#[inline]
fn parse_u64_fast(bytes: &[u8]) -> u64 {
  let mut result = 0_u64;
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
  use microfetch_asm::darwin::macos_meminfo;

  let Some((used_bytes, total_bytes)) = macos_meminfo() else {
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
    let mut total_memory_kb = 0_u64;
    let mut available_memory_kb = 0_u64;
    let mut buffer = [0_u8; 1024];

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

      if let Some(value) = line.strip_prefix(b"MemTotal:") {
        total_memory_kb = parse_u64_fast(value.trim_ascii_start());
        found_total = true;
      }
      if let Some(value) = line.strip_prefix(b"MemAvailable:") {
        available_memory_kb = parse_u64_fast(value.trim_ascii_start());
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
