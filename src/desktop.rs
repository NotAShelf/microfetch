pub fn get_desktop_info() -> String {
    let desktop_env = std::env::var("XDG_CURRENT_DESKTOP");
    let display_backend = std::env::var("XDG_SESSION_TYPE");

    // Trim "none+" from the start of desktop_env if present
    // XXX: This is a workaround for NixOS modules that set XDG_CURRENT_DESKTOP to "none+foo"
    // instead of just "foo"
    // Use "Unknown" if desktop_env or display_backend is empty
    let desktop_env = match desktop_env.as_ref() {
        Err(_) => "Unknown",
        Ok(s) => s.trim_start_matches("none+"),
    };

    let display_backend = display_backend.unwrap_or(String::from("Unknown"));

    format!("{desktop_env} ({display_backend})")
}
