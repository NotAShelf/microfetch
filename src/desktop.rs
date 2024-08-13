pub fn get_desktop_info() -> String {
    fn capitalize_first_letter(s: &str) -> String {
        if s.is_empty() {
            return String::new();
        }

        let mut chars = s.chars();
        let first_char = chars.next().unwrap().to_uppercase().to_string();
        let rest: String = chars.collect();
        first_char + &rest
    }

    // Retrieve the environment variables and handle Result types
    let desktop_env = std::env::var("XDG_CURRENT_DESKTOP");
    let display_backend_result = std::env::var("XDG_SESSION_TYPE");

    // Capitalize the first letter of the display backend value
    let display_backend = capitalize_first_letter(display_backend_result.as_deref().unwrap_or(""));

    // Trim "none+" from the start of desktop_env if present
    // Use "Unknown" if desktop_env is empty or has an error
    let desktop_env = match desktop_env {
        Err(_) => "Unknown".to_string(),
        Ok(s) => s.trim_start_matches("none+").to_string(),
    };

    // Handle the case where display_backend might be empty after capitalization
    let display_backend = if display_backend.is_empty() {
        "Unknown".to_string()
    } else {
        display_backend
    };

    format!("{desktop_env} ({display_backend})")
}
