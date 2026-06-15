use alloc::string::String;

#[cfg(target_os = "linux")]
use crate::syscall::read_file_fast;
use crate::{Error, UtsName};

#[must_use]
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn get_system_info(utsname: &UtsName) -> String {
  let sysname = utsname.sysname().to_str().unwrap_or("Unknown");
  let release = utsname.release().to_str().unwrap_or("Unknown");
  let machine = utsname.machine().to_str().unwrap_or("Unknown");

  // Pre-allocate capacity: sysname + " " + release + " (" + machine + ")"
  let capacity = sysname.len() + 1 + release.len() + 2 + machine.len() + 1;
  let mut result = String::with_capacity(capacity);

  // Manual string construction instead of write! macro
  result.push_str(sysname);
  result.push(' ');
  result.push_str(release);
  result.push_str(" (");
  result.push_str(machine);
  result.push(')');

  result
}

/// Gets the pretty name of the OS via `kern.osproductversion` (macOS),
/// e.g. `macOS 14.5`.
///
/// # Errors
///
/// Never errors; falls back to `macOS` if the version sysctl is unavailable.
#[cfg(target_os = "macos")]
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn get_os_pretty_name() -> Result<String, Error> {
  let mut name = String::from("macOS");
  let mut buf = [0u8; 64];
  if let Some(n) =
    crate::syscall::macos_sysctl_str(b"kern.osproductversion\0", &mut buf)
  {
    if let Ok(ver) = core::str::from_utf8(&buf[..n]) {
      if !ver.is_empty() {
        name.push(' ');
        name.push_str(ver);
      }
    }
  }
  Ok(name)
}

/// Gets the pretty name of the OS from `/etc/os-release`.
///
/// # Errors
///
/// Returns an error if `/etc/os-release` cannot be read.
#[cfg(target_os = "linux")]
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn get_os_pretty_name() -> Result<String, Error> {
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

      // Convert to String - should be valid UTF-8
      return Ok(String::from_utf8_lossy(trimmed).into_owned());
    }

    offset += line_end + 1;
  }

  Ok(String::from("Unknown"))
}
