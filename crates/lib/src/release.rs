#[cfg(target_os = "macos")] use core::str;

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
    && let Ok(version) = str::from_utf8(&buf[..len])
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

/// Writes the hardware model from `hw.model` (macOS), e.g. `Mac14,2`.
#[cfg(target_os = "macos")]
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn write_machine(writer: &mut StackWriter) {
  use microfetch_asm::darwin::macos_sysctl_str;

  let mut buf = [0_u8; 64];
  match macos_sysctl_str(b"hw.model\0", &mut buf) {
    Some(len) if len > 0 => writer.push_bytes(&buf[..len]),
    _ => writer.push_str("Unknown"),
  }
}

/// Writes the hardware model from the DMI product or board name, or the
/// devicetree model on boards without DMI.
#[cfg(target_os = "linux")]
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn write_machine(writer: &mut StackWriter) {
  // https://github.com/fastfetch-cli/fastfetch/blob/2.70.0/src/common/impl/smbios.c#L7-L47
  const PLACEHOLDERS: &[u8] = concat!(
    "*to be filled\0*to be set\0*oem\0*o.e.m.\0*system product\0",
    "*unknown product\0none\0system name\0system version\0default string\0",
    "undefined\0not specified\0not applicable\0not defined\0not available\0",
    "invalid\0type1productconfigid\0all series\0n/a\0unknown\0standard",
  )
  .as_bytes();

  let mut buffer = [0_u8; 128];

  for path in [
    "/sys/devices/virtual/dmi/id/product_name",
    "/sys/devices/virtual/dmi/id/board_name",
    "/sys/firmware/devicetree/base/model",
  ] {
    let Ok(len) = read_file_fast(path, &mut buffer) else {
      continue;
    };
    let raw = &buffer[..len];
    let value = raw.strip_suffix(b"\0").unwrap_or(raw).trim_ascii();

    let placeholder = value.is_empty()
      || PLACEHOLDERS.split(|&byte| byte == 0).any(|entry| {
        let (pattern, is_prefix) = entry
          .strip_prefix(b"*")
          .map_or((entry, false), |pattern| (pattern, true));
        (is_prefix || value.len() == pattern.len())
          && value
            .get(..pattern.len())
            .is_some_and(|head| head.eq_ignore_ascii_case(pattern))
      });

    if !placeholder {
      writer.push_bytes(value);
      return;
    }
  }

  writer.push_str("Unknown");
}
