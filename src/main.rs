mod colors;
mod desktop;
mod release;
mod system;
mod uptime;

use std::io::Write;

use crate::colors::{print_dots, BLUE, CYAN, RESET};
use crate::desktop::get_desktop_info;
use crate::release::{get_os_pretty_name, get_system_info};
use crate::system::{get_memory_usage, get_root_disk_usage, get_shell, get_username_and_hostname};
use crate::uptime::get_current;

use color_eyre::Report;

fn main() -> Result<(), Report> {
    color_eyre::install()?;

    let utsname = nix::sys::utsname::uname()?;
    let fields = Fields {
        user_info: get_username_and_hostname(&utsname),
        os_name: get_os_pretty_name()?,
        kernel_version: get_system_info(&utsname)?,
        shell: get_shell(),
        desktop: get_desktop_info(),
        uptime: get_current()?,
        memory_usage: get_memory_usage()?,
        storage: get_root_disk_usage()?,
        colors: print_dots(),
    };

    print_system_info(&fields);

    Ok(())
}

// Struct to hold all the fields we need to print
// helps avoid clippy warnings about argument count
// and makes it easier to pass around, though its
// not like we need to
struct Fields {
    user_info: String,
    os_name: String,
    kernel_version: String,
    shell: String,
    uptime: String,
    desktop: String,
    memory_usage: String,
    storage: String,
    colors: String,
}

fn print_system_info(fields: &Fields) {
    let Fields {
        user_info,
        os_name,
        kernel_version,
        shell,
        uptime,
        desktop,
        memory_usage,
        storage,
        colors,
    } = fields;

    let _ = std::io::stdout().write_all(format!(
        "
 {CYAN}     ▟█▖    {BLUE}▝█▙ ▗█▛          {user_info} ~{RESET}
 {CYAN}  ▗▄▄▟██▄▄▄▄▄{BLUE}▝█▙█▛  {CYAN}▖        {CYAN}  {BLUE}System{RESET}        {os_name}
 {CYAN}  ▀▀▀▀▀▀▀▀▀▀▀▘{BLUE}▝██  {CYAN}▟█▖       {CYAN}  {BLUE}Kernel{RESET}        {kernel_version}
 {BLUE}     ▟█▛       {BLUE}▝█▘{CYAN}▟█▛        {CYAN}  {BLUE}Shell{RESET}         {shell}
 {BLUE}▟█████▛          {CYAN}▟█████▛     {CYAN}  {BLUE}Uptime{RESET}        {uptime}
 {BLUE}   ▟█▛{CYAN}▗█▖       {CYAN}▟█▛          {CYAN}  {BLUE}Desktop{RESET}       {desktop}
 {BLUE}  ▝█▛  {CYAN}██▖{BLUE}▗▄▄▄▄▄▄▄▄▄▄▄       {CYAN}󰍛  {BLUE}Memory{RESET}        {memory_usage}
 {BLUE}   ▝  {CYAN}▟█▜█▖{BLUE}▀▀▀▀▀██▛▀▀▘       {CYAN}󱥎  {BLUE}Storage (/){RESET}   {storage}
 {CYAN}     ▟█▘ ▜█▖    {BLUE}▝█▛          {CYAN}  {BLUE}Colors{RESET}        {colors}
").as_bytes());
}
