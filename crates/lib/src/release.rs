#[cfg(target_os = "linux")]
use microfetch_asm::read_file_fast;

use crate::{StackWriter, UtsName};

#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn write_system_info(writer: &mut StackWriter, utsname: &UtsName) {
  writer.push_cstr(utsname.sysname());
  writer.push_byte(b' ');
  writer.push_cstr(utsname.release());
  writer.push_str(" (");
  writer.push_cstr(utsname.machine());
  writer.push_byte(b')');
}

/// Writes the OS name from `kern.osproductversion` (macOS), e.g. `macOS 14.5`.
/// Falls back to `macOS` if the version sysctl is unavailable.
#[cfg(target_os = "macos")]
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn write_os_pretty_name(writer: &mut StackWriter) {
  use microfetch_asm::darwin::macos_sysctl_str;

  writer.push_str("macOS");
  let mut buf = [0_u8; 64];
  if let Some(len) = macos_sysctl_str(b"kern.osproductversion\0", &mut buf)
    && let Ok(version) = core::str::from_utf8(&buf[..len])
    && !version.is_empty()
  {
    writer.push_byte(b' ');
    writer.push_str(version);
  }
}

/// Writes the pretty name of the OS from `/etc/os-release`, or nothing if it
/// cannot be read.
#[cfg(target_os = "linux")]
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn write_os_pretty_name(writer: &mut StackWriter) {
  // Fast byte-level scanning for PRETTY_NAME=
  const PREFIX: &[u8] = b"PRETTY_NAME=";

  let mut buffer = [0_u8; 1024];

  // Use fast syscall-based file reading
  let Ok(bytes_read) = read_file_fast("/etc/os-release", &mut buffer) else {
    return;
  };
  let content = &buffer[..bytes_read];
  let mut offset = 0;

  while offset < content.len() {
    let remaining = &content[offset..];

    // Find newline or end
    let line_end = remaining
      .iter()
      .position(|&byte| byte == b'\n')
      .unwrap_or(remaining.len());
    let line = &remaining[..line_end];

    if line.starts_with(PREFIX) {
      let value = &line[PREFIX.len()..];

      // Strip quotes if present
      let trimmed = if value.len() >= 2
        && value[0] == b'"'
        && value[value.len() - 1] == b'"'
      {
        &value[1..value.len() - 1]
      } else {
        value
      };

      // Write bytes directly — os-release values are effectively always
      // valid UTF-8 (ASCII names, version numbers). No lossy conversion
      // needed.
      writer.push_bytes(trimmed);
      return;
    }

    offset += line_end + 1;
  }

  writer.push_str("Unknown");
}
