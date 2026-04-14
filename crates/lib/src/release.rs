#[cfg(target_os = "linux")]
use crate::syscall::read_file_fast;
use crate::{Error, StackWriter, UtsName};

#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn write_system_info(w: &mut StackWriter, utsname: &UtsName) {
  w.push_cstr(utsname.sysname());
  w.push_byte(b' ');
  w.push_cstr(utsname.release());
  w.push_str(" (");
  w.push_cstr(utsname.machine());
  w.push_byte(b')');
}

/// Writes the OS name from `kern.osproductversion` (macOS), e.g. `macOS 14.5`.
///
/// # Errors
///
/// Never errors; falls back to `macOS` if the version sysctl is unavailable.
#[cfg(target_os = "macos")]
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn write_os_pretty_name(w: &mut StackWriter) -> Result<(), Error> {
  w.push_str("macOS");
  let mut buf = [0u8; 64];
  if let Some(n) =
    crate::syscall::macos_sysctl_str(b"kern.osproductversion\0", &mut buf)
  {
    if let Ok(version) = core::str::from_utf8(&buf[..n]) {
      if !version.is_empty() {
        w.push_byte(b' ');
        w.push_str(version);
      }
    }
  }
  Ok(())
}

/// Writes the pretty name of the OS from `/etc/os-release`.
///
/// # Errors
///
/// Returns an error if `/etc/os-release` cannot be read.
#[cfg(target_os = "linux")]
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn write_os_pretty_name(w: &mut StackWriter) -> Result<(), Error> {
  // Fast byte-level scanning for PRETTY_NAME=
  const PREFIX: &[u8] = b"PRETTY_NAME=";

  let mut buffer = [0u8; 1024];

  // Use fast syscall-based file reading
  let bytes_read = read_file_fast("/etc/os-release", &mut buffer)
    .map_err(Error::from_raw_os_error)?;
  let content = &buffer[..bytes_read];
  let mut offset = 0;

  while offset < content.len() {
    let remaining = &content[offset..];

    // Find newline or end
    let line_end = remaining
      .iter()
      .position(|&b| b == b'\n')
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
      w.push_bytes(trimmed);
      return Ok(());
    }

    offset += line_end + 1;
  }

  w.push_str("Unknown");
  Ok(())
}
