use color_eyre::Result;
use std::{env, io};

pub fn get_desktop_info() -> Result<String, io::Error> {
    let desktop_env = env::var("XDG_CURRENT_DESKTOP");
    let display_backend = env::var("XDG_SESSION_TYPE");

    // Trim "none+" from the start of desktop_env if present
    // XXX: This is a workaround for NixOS modules that set XDG_CURRENT_DESKTOP to "none+foo"
    // instead of just "foo"
    // Use "Unknown" if desktop_env or display_backend is empty
    let desktop_env = match desktop_env {
        Err(_) => String::from("Unknown"),
        Ok(s) => s.trim_start_matches("none+").to_owned(),
    };

    let display_backend = display_backend.unwrap_or_else(|_| String::from("Unknown"));

    Ok(format!("{desktop_env} ({display_backend})"))
}
