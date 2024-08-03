use std::{env, io};

pub fn get_desktop_info() -> Result<String, io::Error> {
    let desktop_env = env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
    let display_backend = env::var("XDG_SESSION_TYPE").unwrap_or_default();

    // Trim "none+" from the start of desktop_env if present
    // XXX: This is a workaround for NixOS modules that set XDG_CURRENT_DESKTOP to "none+foo"
    // instead of just "foo"
    let desktop_env = desktop_env.trim_start_matches("none+");

    Ok(format!("{} ({})", desktop_env, display_backend))
}
