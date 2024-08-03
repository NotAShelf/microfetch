mod colors;
mod desktop;
mod release;
mod system;
mod uptime;

use color_eyre::{Report, Result};

use crate::colors::{BLUE, CYAN, RESET};
use crate::desktop::get_desktop_info;
use crate::release::{get_os_pretty_name, get_system_info};
use crate::system::{get_disk_usage, get_memory_usage, get_username_and_hostname};
use crate::uptime::get_system_uptime;

fn main() -> Result<(), Report> {
    color_eyre::install()?;

    let user_info = get_username_and_hostname().expect("Failed to get username and hostname");
    let os_name = get_os_pretty_name().expect("Failed to get OS name");
    let kernel_version = get_system_info().expect("Failed to get kernel info");
    let uptime = get_system_uptime().expect("Failed to get uptime");
    let window_manager = get_desktop_info().expect("Failed to get desktop info");
    let memory_usage = get_memory_usage().expect("Failed to get memory usage");
    let storage = get_disk_usage().expect("Failed to get storage info");

    // Construct the ASCII art with dynamic OS name

    println!(
        "
{CYAN}  ▗▄   {BLUE}▗▄ ▄▖         {} ~{RESET}
{CYAN} ▄▄🬸█▄▄▄{BLUE}🬸█▛ {CYAN}▃        {CYAN}  {BLUE}System{RESET}    {}
{BLUE}   ▟▛    ▜{CYAN}▃▟🬕        {CYAN}  {BLUE}Kernel{RESET}    {}
{BLUE}🬋🬋🬫█      {CYAN}█🬛🬋🬋       {CYAN}  {BLUE}Uptime{RESET}    {}
{BLUE} 🬷▛🮃{CYAN}▙    ▟▛          {CYAN}  {BLUE}WM{RESET}        {}
{BLUE} 🮃 {CYAN}▟█🬴{BLUE}▀▀▀█🬴▀▀        {CYAN}󰍛  {BLUE}Memory{RESET}    {}
{CYAN}  ▝▀ ▀▘   {BLUE}▀▘         {CYAN}󱥎  {BLUE}Storage{RESET}   {}
    ",
        user_info, os_name, kernel_version, uptime, window_manager, memory_usage, storage
    );

    Ok(())
}
