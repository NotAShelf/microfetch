use crate::{StackWriter, getenv_str};

#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn write_desktop_info(writer: &mut StackWriter) {
  let desktop_raw = getenv_str("XDG_CURRENT_DESKTOP");
  let session_raw = getenv_str("XDG_SESSION_TYPE");

  let desktop_str = desktop_raw.map_or_else(
    || {
      if cfg!(target_os = "macos") {
        "Aqua"
      } else {
        "Unknown"
      }
    },
    |desktop| desktop.strip_prefix("none+").unwrap_or(desktop),
  );

  let backend_str = session_raw.unwrap_or({
    if cfg!(target_os = "macos") {
      "Quartz"
    } else {
      "Unknown"
    }
  });

  writer.push_str(desktop_str);
  writer.push_str(" (");

  // Capitalize first character of backend
  if let Some(&first_byte) = backend_str.as_bytes().first() {
    // Convert first byte to uppercase if it's ASCII lowercase
    let upper = if first_byte.is_ascii_lowercase() {
      first_byte - b'a' + b'A'
    } else {
      first_byte
    };
    writer.push_byte(upper);
    writer.push_str(&backend_str[1..]);
  }

  writer.push_byte(b')');
}
