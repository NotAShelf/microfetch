mod colors;
mod desktop;
mod release;
mod system;
mod uptime;

use crate::colors::{print_dots, BLUE, CYAN, RESET};
use crate::desktop::get_desktop_info;
use crate::release::{get_os_pretty_name, get_system_info};
use crate::system::{get_memory_usage, get_root_disk_usage, get_shell, get_username_and_hostname};
use crate::uptime::get_current;
use color_eyre::Report;
use std::io::{self, Write};

fn main() -> Result<(), Report> {
    color_eyre::install()?;

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--version" {
        println!("Microfetch {}", env!("CARGO_PKG_VERSION"));
    } else {
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
        print_system_info(&fields)?;
    }

    Ok(())
}

// Struct to hold all the fields we need to print
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

const LOGO: [&str; 9] = [
    "     ▟█▖    ▝█▙ ▗█▛",
    "  ▗▄▄▟██▄▄▄▄▄▝█▙█▛  ▖",
    "  ▀▀▀▀▀▀▀▀▀▀▀▘▝██  ▟█▖",
    "     ▟█▛       ▝█▘▟█▛",
    "▟█████▛          ▟█████▛",
    "   ▟█▛▗█▖       ▟█▛",
    "  ▝█▛  ██▖▗▄▄▄▄▄▄▄▄▄▄▄",
    "   ▝  ▟█▜█▖▀▀▀▀▀██▛▀▀▘",
    "     ▟█▘ ▜█▖    ▝█▛ ",
];

fn print_system_info(fields: &Fields) -> io::Result<()> {
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

    // Labels and values to be printed
    let system_info = [
        ("", "System", os_name),
        ("", "Kernel", kernel_version),
        ("", "Shell", shell),
        ("", "Uptime", uptime),
        ("", "Desktop", desktop),
        ("󰍛", "Memory", memory_usage),
        ("󱥎", "Storage (/)", storage),
        ("", "Colors", colors),
    ];

    let label_width = system_info
        .iter()
        .map(|(_, label, _)| label.len())
        .max()
        .unwrap_or(0);

    writeln!(io::stdout(), "{:<27} {} ~{RESET}", LOGO[0], user_info)?;
    for (logo_line, (icon, label, value)) in LOGO[1..].iter().zip(system_info.iter()) {
        writeln!(
            io::stdout(),
            "{:<27} {CYAN}{:<2} {BLUE}{:<width$}{RESET}        {value}{RESET}",
            logo_line,
            icon,
            label,
            width = label_width
        )?;
    }

    Ok(())
}
