mod colors;
mod desktop;
mod release;
mod system;
mod uptime;

use nix::sys::sysinfo::sysinfo;

use crate::colors::{BLUE, CYAN, RESET};
use crate::desktop::get_desktop_info;
use crate::release::{get_os_pretty_name, get_system_info};
use crate::system::{get_memory_usage, get_root_disk_usage, get_username_and_hostname};
use crate::uptime::get_current;

fn main() -> Result<(), Report> {
    color_eyre::install()?;

    let user_info = get_username_and_hostname()?;
    let os_name = get_os_pretty_name()?;
    let kernel_version = get_system_info()?;
    let uptime = get_current()?;
    let window_manager = get_desktop_info()?;
    let memory_usage = get_memory_usage()?;
    let storage = get_root_disk_usage()?;

    print_system_info(
        &user_info,
        &os_name,
        &kernel_version,
        &uptime,
        &window_manager,
        &memory_usage,
        &storage,
    );
}

fn print_system_info(
    user_info: &str,
    os_name: &str,
    kernel_version: &str,
    uptime: &str,
    window_manager: &str,
    memory_usage: &str,
    storage: &str,
) {
    println!(
        "
{CYAN}   ▜▙▖  {BLUE}▝▜▙ ▟█▘       {user_info} ~{RESET}
{CYAN} ▗▟█████▙▖{BLUE}▜██▘ {CYAN}▟▙     {CYAN}  {BLUE}System{RESET}        {os_name}
{BLUE}    ▄▖     ▜▛ {CYAN}▟▛▘     {CYAN}  {BLUE}Kernel{RESET}        {kernel_version}
{BLUE}▟███▛ {CYAN}▖    {BLUE}▝ {CYAN}▟███▛    {CYAN}  {BLUE}Uptime{RESET}        {uptime}
{BLUE} ▗▟▛ {CYAN}▟▙     ▝▀        {CYAN}  {BLUE}WM{RESET}            {window_manager}
{BLUE} ▜▛ {CYAN}▗██▙{BLUE}▝▜█████▛▘     {CYAN}󰍛  {BLUE}Memory{RESET}        {memory_usage}
{CYAN}   ▗█▛ ▜▙▖  {BLUE}▝▜▙       {CYAN}󱥎  {BLUE}Storage (/){RESET}   {storage}


 {CYAN}     ▟█▖    {BLUE}▝█▙ ▗█▛          {user_info} ~{RESET}
 {CYAN}  ▗▄▄▟██▄▄▄▄▄{BLUE}▝█▙█▛  {CYAN}▖        {CYAN}  {BLUE}System{RESET}        {os_name}
 {CYAN}  ▀▀▀▀▀▀▀▀▀▀▀▘{BLUE}▝██  {CYAN}▟█▖       {CYAN}  {BLUE}Kernel{RESET}        {kernel_version}
 {BLUE}     ▟█▛       {BLUE}▝█▘{CYAN}▟█▛        {CYAN}  {BLUE}Uptime{RESET}        {uptime}
 {BLUE}▟█████▛          {CYAN}▟█████▛     {CYAN}  {BLUE}WM{RESET}            {window_manager}
 {BLUE}   ▟█▛{CYAN}▗█▖       {CYAN}▟█▛          {CYAN}󰍛  {BLUE}Memory{RESET}        {memory_usage}
 {BLUE}  ▝█▛  {CYAN}██▖{BLUE}▗▄▄▄▄▄▄▄▄▄▄▄       {CYAN}󱥎  {BLUE}Storage (/){RESET}   {storage}
 {BLUE}   ▝  {CYAN}▟█▜█▖{BLUE}▀▀▀▀▀██▛▀▀▘              
 {CYAN}     ▟█▘ ▜█▖    {BLUE}▝█▛                 
    "
    );
}
