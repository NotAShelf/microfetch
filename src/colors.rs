pub const RESET: &str = "\x1b[0m";
pub const BLUE: &str = "\x1b[34m";
pub const CYAN: &str = "\x1b[36m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const RED: &str = "\x1b[31m";
pub const MAGENTA: &str = "\x1b[35m";

pub fn print_dots() -> Result<String, std::io::Error> {
    let colors = format!("{BLUE}  {CYAN}  {GREEN}  {YELLOW}  {RED}  {MAGENTA}  {RESET}");

    Ok(colors)
}
