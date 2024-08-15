pub fn get_desktop_info() -> String {
    // Retrieve the environment variables and handle Result types
    let desktop_env = std::env::var("XDG_CURRENT_DESKTOP");
    let display_backend_result = std::env::var("XDG_SESSION_TYPE");

    // Capitalize the first letter of the display backend value
    let mut display_backend = display_backend_result.unwrap_or_default();
    if let Some(c) = display_backend.as_mut_str().get_mut(0..1) {
        c.make_ascii_uppercase();
    }

    // Trim "none+" from the start of desktop_env if present
    // Use "Unknown" if desktop_env is empty or has an error
    let desktop_env = match desktop_env {
        Err(_) => "Unknown".to_string(),
        Ok(s) => s.trim_start_matches("none+").to_string(),
    };

    // Handle the case where display_backend might be empty after capitalization
    let display_backend = if display_backend.is_empty() {
        "Unknown"
    } else {
        &display_backend
    }
    .to_string();

    format!("{desktop_env} ({display_backend})")
}
