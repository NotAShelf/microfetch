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

/// RAM and swap usage in bytes.
pub struct Memory {
  used:       u64,
  total:      u64,
  swap_used:  u64,
  swap_total: u64,
}

impl Memory {
  /// Reads RAM and swap usage via `sysctl`/Mach (macOS).
  #[cfg(target_os = "macos")]
  #[cfg_attr(feature = "hotpath", hotpath::measure)]
  #[must_use = "this only queries the kernel"]
  pub fn read() -> Option<Self> {
    use microfetch_asm::darwin::{macos_meminfo, macos_swapinfo};

    let (used, total) = macos_meminfo()?;
    let (swap_used, swap_total) = macos_swapinfo().unwrap_or((0, 0));
    Some(Self {
      used,
      total,
      swap_used,
      swap_total,
    })
  }

  /// Reads RAM and swap usage from `/proc/meminfo`.
  #[cfg(target_os = "linux")]
  #[cfg_attr(feature = "hotpath", hotpath::measure)]
  #[must_use = "this only reads /proc/meminfo"]
  pub fn read() -> Option<Self> {
    let mut buffer = [0_u8; 1024];
    let bytes_read = read_file_fast("/proc/meminfo", &mut buffer).ok()?;

    let mut mem_total = 0_u64;
    let mut mem_available = 0_u64;
    let mut swap_total = 0_u64;
    let mut swap_free = 0_u64;

    for line in buffer[..bytes_read].split(|&byte| byte == b'\n') {
      let (field, rest) = if let Some(rest) = line.strip_prefix(b"MemTotal:") {
        (&mut mem_total, rest)
      } else if let Some(rest) = line.strip_prefix(b"MemAvailable:") {
        (&mut mem_available, rest)
      } else if let Some(rest) = line.strip_prefix(b"SwapTotal:") {
        (&mut swap_total, rest)
      } else if let Some(rest) = line.strip_prefix(b"SwapFree:") {
        swap_free = parse_u64_fast(rest.trim_ascii_start());
        break;
      } else {
        continue;
      };
      *field = parse_u64_fast(rest.trim_ascii_start());
    }

    Some(Self {
      used:       (mem_total - mem_available) << 10,
      total:      mem_total << 10,
      swap_used:  (swap_total - swap_free) << 10,
      swap_total: swap_total << 10,
    })
  }
}

fn write_usage(
  writer: &mut StackWriter,
  colors: &Colors,
  used: u64,
  total: u64,
) {
  let percentage_used =
    (used * 100 + total / 2).checked_div(total).unwrap_or(0);

  write_gib(writer, used);
  writer.push_str(" GiB / ");
  write_gib(writer, total);
  writer.push_str(" GiB (");
  writer.push_str(colors.cyan);
  writer.push_u64(percentage_used);
  writer.push_byte(b'%');
  writer.push_str(colors.reset);
  writer.push_byte(b')');
}

/// Writes RAM usage, or nothing if it could not be read.
pub fn write_memory_usage(
  writer: &mut StackWriter,
  colors: &Colors,
  memory: Option<&Memory>,
) {
  let Some(usage) = memory else {
    return;
  };
  write_usage(writer, colors, usage.used, usage.total);
}

/// Writes swap usage, `Disabled` when there is no swap, or nothing if it could
/// not be read.
pub fn write_swap_usage(
  writer: &mut StackWriter,
  colors: &Colors,
  memory: Option<&Memory>,
) {
  let Some(usage) = memory else {
    return;
  };
  if usage.swap_total == 0 {
    writer.push_str("Disabled");
    return;
  }
  write_usage(writer, colors, usage.swap_used, usage.swap_total);
}
