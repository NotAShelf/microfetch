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
  if let Some((&first_byte, rest)) = backend_str.as_bytes().split_first() {
    writer.push_byte(first_byte.to_ascii_uppercase());
    writer.push_bytes(rest);
  }

  writer.push_byte(b')');
}
