#![no_std]
extern crate alloc;

pub mod colors;
pub mod cpu;
pub mod desktop;
pub mod release;
pub mod system;
pub mod uptime;

use alloc::string::String;
use core::{
  ffi::CStr,
  mem::MaybeUninit,
  sync::atomic::{AtomicPtr, Ordering},
};

pub use microfetch_asm as syscall;
pub use microfetch_asm::{
  StatfsBuf,
  SysInfo,
  UtsNameBuf,
  read_file_fast,
  sys_close,
  sys_open,
  sys_read,
  sys_sched_getaffinity,
  sys_statfs,
  sys_sysinfo,
  sys_uname,
  sys_write,
};

/// A simple error type for microfetch operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
  /// An OS error occurred, containing the errno value.
  OsError(i32),
  /// Invalid data or encoding error.
  InvalidData,
  /// Not found.
  NotFound,
  /// Write operation failed or partial write.
  WriteError,
}

impl Error {
  /// Creates an error from the last OS error (reads errno).
  #[inline]
  #[must_use]
  pub const fn last_os_error() -> Self {
    // This is a simplified version - in a real implementation,
    // we'd need to get the actual errno from the syscall return
    Self::OsError(0)
  }

  /// Creates an error from a raw OS error code (negative errno from syscall).
  #[inline]
  #[must_use]
  pub const fn from_raw_os_error(errno: i32) -> Self {
    Self::OsError(-errno)
  }
}

impl core::fmt::Display for Error {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::OsError(errno) => write!(f, "OS error: {errno}"),
      Self::InvalidData => write!(f, "Invalid data"),
      Self::NotFound => write!(f, "Not found"),
      Self::WriteError => write!(f, "Write error"),
    }
  }
}

// Simple OnceLock implementation for no_std
pub struct OnceLock<T> {
  ptr: AtomicPtr<T>,
}

impl<T> Default for OnceLock<T> {
  fn default() -> Self {
    Self::new()
  }
}

impl<T> OnceLock<T> {
  #[must_use]
  pub const fn new() -> Self {
    Self {
      ptr: AtomicPtr::new(core::ptr::null_mut()),
    }
  }

  pub fn get_or_init<F>(&self, f: F) -> &T
  where
    F: FnOnce() -> T,
  {
    // Load the current pointer
    let mut ptr = self.ptr.load(Ordering::Acquire);

    if ptr.is_null() {
      // Need to initialize
      let value = f();
      let boxed = alloc::boxed::Box::new(value);
      let new_ptr = alloc::boxed::Box::into_raw(boxed);

      // Try to set the pointer
      match self.ptr.compare_exchange(
        core::ptr::null_mut(),
        new_ptr,
        Ordering::Release,
        Ordering::Acquire,
      ) {
        Ok(_) => {
          // We successfully set it
          ptr = new_ptr;
        },
        Err(existing) => {
          // Someone else set it first, free our allocation
          // SAFETY: We just allocated this and no one else has seen it
          unsafe {
            let _ = alloc::boxed::Box::from_raw(new_ptr);
          }
          ptr = existing;
        },
      }
    }

    // SAFETY: We know ptr is non-null and points to a valid T
    unsafe { &*ptr }
  }
}

impl<T> Drop for OnceLock<T> {
  fn drop(&mut self) {
    let ptr = self.ptr.load(Ordering::Acquire);
    if !ptr.is_null() {
      // SAFETY: We know this was allocated via Box::into_raw
      unsafe {
        let _ = alloc::boxed::Box::from_raw(ptr);
      }
    }
  }
}

// Store the environment pointer internally,initialized from `main()`. This
// helps avoid the libc dependency *completely*.
static ENVP: AtomicPtr<*const u8> = AtomicPtr::new(core::ptr::null_mut());

/// Initialize the environment pointer. Must be called before any `getenv()`
/// calls. This is called from `main()` with the calculated `envp`.
///
/// # Safety
///
/// envp must be a valid null-terminated array of C strings, or null if
/// no environment is available.
#[inline]
pub unsafe fn init_env(envp: *const *const u8) {
  ENVP.store(envp.cast_mut(), Ordering::Release);
}

/// Gets the current environment pointer.
#[inline]
#[must_use]
fn get_envp() -> *const *const u8 {
  ENVP.load(Ordering::Acquire)
}

/// Gets an environment variable by name without using std or libc by reading
/// from the environment pointer set by [`init_env`].
#[must_use]
pub fn getenv(name: &str) -> Option<&'static [u8]> {
  let envp = get_envp();
  if envp.is_null() {
    return None;
  }

  let name_bytes = name.as_bytes();

  // Walk through environment variables
  let mut i = 0;
  loop {
    // SAFETY: environ is null-terminated array of pointers
    let entry = unsafe { *envp.add(i) };
    if entry.is_null() {
      break;
    }

    // Check if this entry starts with our variable name followed by '='
    let mut matches = true;
    for (j, &b) in name_bytes.iter().enumerate() {
      // SAFETY: entry is a valid C string
      let entry_byte = unsafe { *entry.add(j) };
      if entry_byte != b {
        matches = false;
        break;
      }
    }

    if matches {
      // Check for '=' after the name
      // SAFETY: entry is a valid C string
      let eq_byte = unsafe { *entry.add(name_bytes.len()) };
      if eq_byte == b'=' {
        // Found it! Calculate the value length
        let value_start = unsafe { entry.add(name_bytes.len() + 1) };
        let mut len = 0;
        loop {
          // SAFETY: entry is a valid C string
          let b = unsafe { *value_start.add(len) };
          if b == 0 {
            break;
          }
          len += 1;
        }
        // SAFETY: We calculated the exact length
        return Some(unsafe { core::slice::from_raw_parts(value_start, len) });
      }
    }

    i += 1;
  }

  None
}

/// Gets an environment variable as a UTF-8 string.
#[must_use]
pub fn getenv_str(name: &str) -> Option<&'static str> {
  getenv(name).and_then(|bytes| core::str::from_utf8(bytes).ok())
}

/// Checks if an environment variable exists (regardless of its value).
#[must_use]
pub fn env_exists(name: &str) -> bool {
  getenv(name).is_some()
}

/// Wrapper for `utsname` with safe accessor methods
pub struct UtsName(UtsNameBuf);

impl UtsName {
  /// Calls `uname(2)` syscall and returns a `UtsName` wrapper
  ///
  /// # Errors
  ///
  /// Returns an error if the `uname` syscall fails
  pub fn uname() -> Result<Self, Error> {
    let mut uts = MaybeUninit::uninit();
    if unsafe { sys_uname(uts.as_mut_ptr()) } != 0 {
      return Err(Error::last_os_error());
    }

    Ok(Self(unsafe { uts.assume_init() }))
  }

  #[must_use]
  pub const fn nodename(&self) -> &CStr {
    unsafe { CStr::from_ptr(self.0.nodename.as_ptr().cast()) }
  }

  #[must_use]
  pub const fn sysname(&self) -> &CStr {
    unsafe { CStr::from_ptr(self.0.sysname.as_ptr().cast()) }
  }

  #[must_use]
  pub const fn release(&self) -> &CStr {
    unsafe { CStr::from_ptr(self.0.release.as_ptr().cast()) }
  }

  #[must_use]
  pub const fn machine(&self) -> &CStr {
    unsafe { CStr::from_ptr(self.0.machine.as_ptr().cast()) }
  }
}

// Struct to hold all the fields we need in order to print the fetch. This
// helps avoid Clippy warnings about argument count, and makes it slightly
// easier to pass data around. Though, it is not like we really need to.
struct Fields {
  user_info:      String,
  os_name:        String,
  kernel_version: String,
  cpu_name:       String,
  cpu_cores:      String,
  shell:          String,
  uptime:         String,
  desktop:        String,
  memory_usage:   String,
  storage:        String,
  colors:         String,
}

/// Minimal, stack-allocated writer implementing `core::fmt::Write`. Avoids heap
/// allocation for the output buffer.
struct StackWriter<'a> {
  buf: &'a mut [u8],
  pos: usize,
}

impl<'a> StackWriter<'a> {
  #[inline]
  const fn new(buf: &'a mut [u8]) -> Self {
    Self { buf, pos: 0 }
  }

  #[inline]
  fn written(&self) -> &[u8] {
    &self.buf[..self.pos]
  }
}

impl core::fmt::Write for StackWriter<'_> {
  #[inline]
  fn write_str(&mut self, s: &str) -> core::fmt::Result {
    let bytes = s.as_bytes();
    let to_write = bytes.len().min(self.buf.len() - self.pos);
    self.buf[self.pos..self.pos + to_write].copy_from_slice(&bytes[..to_write]);
    self.pos += to_write;
    Ok(())
  }
}

/// Custom logo art embedded at compile time via the `MICROFETCH_LOGO`
/// environment variable. Set it to 11 newline-separated lines of ASCII/Unicode
/// art when building to replace the default NixOS logo:
///
///   `MICROFETCH_LOGO="$(cat my_logo.txt)"` cargo build --release
///
/// Each line maps to one info row. When unset, the built-in two-tone NixOS
/// logo is used.
const CUSTOM_LOGO: &str = match option_env!("MICROFETCH_LOGO") {
  Some(s) => s,
  None => "",
};

#[cfg_attr(feature = "hotpath", hotpath::measure)]
fn print_system_info(fields: &Fields) -> Result<(), Error> {
  let Fields {
    user_info,
    os_name,
    kernel_version,
    cpu_name,
    cpu_cores,
    shell,
    uptime,
    desktop,
    memory_usage,
    storage,
    colors,
  } = fields;

  let no_color = colors::is_no_color();
  let c = colors::Colors::new(no_color);

  let mut buf = [0u8; 2560];
  let mut w = StackWriter::new(&mut buf);

  if CUSTOM_LOGO.is_empty() {
    // Default two-tone NixOS logo rendered as a single write! pass.
    core::fmt::write(
      &mut w,
      format_args!(
        "\n    {l1}⠀⠀⠀⠀⠀⠀⢼⣿⣄⠀⠀⠀{l2}⠹⣿⣷⡀⠀⣠⣿⡧⠀⠀⠀⠀⠀⠀{rs}  {user_info} ~{rs}\
         \n    {l1}⠀⠀⠀⠀⠀⠀⠈⢿⣿⣆⠀⠀⠀{l2}⠘⣿⣿⣴⣿⡿⠁⠀⠀⠀⠀⠀⠀{rs}  {icon}\u{F313}  {key}System{rs}       \u{E621} {val}{os_name}{rs}\
         \n    {l1}⠀⠀⠀⢠⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⡜{l2}⢿⣿⣟⠀⠀⠀{l1}⢀⡄⠀⠀⠀{rs}  {icon}\u{E712}  {key}Kernel{rs}       \u{E621} {val}{kernel_version}{rs}\
         \n    {l1}⠀⠀⠀⠉⠉⠉⠉{l2}⣩⣭⡭{l1}⠉⠉⠉⠉⠉{l2}⠈⢿⣿⣆⠀{l1}⢠⣿⣿⠂⠀⠀{rs}  {icon}\u{F2DB}  {key}CPU{rs}          \u{E621} {val}{cpu_name}{rs}\
         \n    {l2}⠀⠀⠀⠀⠀⠀⣼⣿⡟⠀⠀⠀⠀⠀⠀⠀⠀⢻⡟{l1}⣡⣿⣿⠃⠀⠀⠀{rs}  {icon}\u{F4BC}  {key}Topology{rs}     \u{E621} {val}{cpu_cores}{rs}\
         \n    {l2}⢸⣿⣿⣿⣿⣿⣿⠏⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀{l1}⣰⣿⣿⣿⣿⣿⣿⡇{rs}  {icon}\u{E795}  {key}Shell{rs}        \u{E621} {val}{shell}{rs}\
         \n    {l2}⠀⠀⠀⢠⣿⣿⢋{l1}⣼⣧⠀⠀⠀⠀⠀⠀⠀⠀⣼⣿⡟⠀⠀⠀⠀⠀⠀{rs}  {icon}\u{F017}  {key}Uptime{rs}       \u{E621} {val}{uptime}{rs}\
         \n    {l2}⠀⠀⠠⣿⣿⠃⠀{l1}⠹⣿⣷⡀{l2}⣀⣀⣀⣀⣀{l1}⣚⣛⣋{l2}⣀⣀⣀⣀⠀⠀⠀{rs}  {icon}\u{F2D2}  {key}Desktop{rs}      \u{E621} {val}{desktop}{rs}\
         \n    {l2}⠀⠀⠀⠘⠁⠀⠀⠀{l1}⣽⣿⣷⡜{l2}⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠃⠀⠀⠀{rs}  {icon}\u{F035B}  {key}Memory{rs}       \u{E621} {val}{memory_usage}{rs}\
         \n    {l1}⠀⠀⠀⠀⠀⠀⢀⣾⣿⠟⣿⣿⡄⠀⠀⠀{l2}⠹⣿⣷⡀⠀⠀⠀⠀⠀⠀{rs}  {icon}\u{F194E}  {key}Storage (/){rs}  \u{E621} {val}{storage}{rs}\
         \n    {l1}⠀⠀⠀⠀⠀⠀⢺⣿⠋⠀⠈⢿⣿⣆⠀⠀⠀{l2}⠙⣿⡗⠀⠀⠀⠀⠀⠀{rs}  {icon}\u{E22B}  {key}Colors{rs}       \u{E621} {val}{colors}{rs}\n\n",
        l1 = c.l1,
        l2 = c.l2,
        icon = c.icon,
        key = c.key,
        val = c.value,
        rs = c.reset,
        user_info = user_info,
        os_name = os_name,
        kernel_version = kernel_version,
        cpu_name = cpu_name,
        cpu_cores = cpu_cores,
        shell = shell,
        uptime = uptime,
        desktop = desktop,
        memory_usage = memory_usage,
        storage = storage,
        colors = colors,
      ),
    )
    .ok();
  } else {
    // Custom logo is 11 lines from MICROFETCH_LOGO env var, one per info row.
    // Lines beyond 11 are ignored; missing lines render as empty.
    let mut lines = CUSTOM_LOGO.split('\n');
    let logo_rows: [&str; 11] =
      core::array::from_fn(|_| lines.next().unwrap_or(""));

    // Row format mirrors the default logo path exactly.
    let rows: [(&str, &str, &str, &str, &str); 11] = [
      ("", "", user_info.as_str(), "        ", " ~"),
      (
        "\u{F313}  ",
        "System",
        os_name.as_str(),
        "       \u{E621} ",
        "",
      ),
      (
        "\u{E712}  ",
        "Kernel",
        kernel_version.as_str(),
        "       \u{E621} ",
        "",
      ),
      (
        "\u{F2DB}  ",
        "CPU",
        cpu_name.as_str(),
        "          \u{E621} ",
        "",
      ),
      (
        "\u{F4BC}  ",
        "Topology",
        cpu_cores.as_str(),
        "     \u{E621} ",
        "",
      ),
      (
        "\u{E795}  ",
        "Shell",
        shell.as_str(),
        "        \u{E621} ",
        "",
      ),
      (
        "\u{F017}  ",
        "Uptime",
        uptime.as_str(),
        "       \u{E621} ",
        "",
      ),
      (
        "\u{F2D2}  ",
        "Desktop",
        desktop.as_str(),
        "      \u{E621} ",
        "",
      ),
      (
        "\u{F035B}  ",
        "Memory",
        memory_usage.as_str(),
        "       \u{E621} ",
        "",
      ),
      (
        "\u{F194E}  ",
        "Storage (/)",
        storage.as_str(),
        "  \u{E621} ",
        "",
      ),
      (
        "\u{E22B}  ",
        "Colors",
        colors.as_str(),
        "       \u{E621} ",
        "",
      ),
    ];

    core::fmt::write(&mut w, format_args!("\n")).ok();
    for i in 0..11 {
      let (icon, key, value, spacing, suffix) = rows[i];
      if key.is_empty() {
        // Row 1 has  no icon/key, just logo + user_info
        core::fmt::write(
          &mut w,
          format_args!(
            "    {cy}{logo}{rs}  {value}{suffix}\n",
            cy = c.cyan,
            rs = c.reset,
            logo = logo_rows[i],
            value = value,
            suffix = suffix,
          ),
        )
        .ok();
      } else {
        core::fmt::write(
          &mut w,
          format_args!(
            "    {cy}{logo}{rs}  \
             {cy}{icon}{b}{key}{rs}{spacing}{value}{suffix}\n",
            cy = c.cyan,
            b = c.blue,
            rs = c.reset,
            logo = logo_rows[i],
            icon = icon,
            key = key,
            spacing = spacing,
            value = value,
            suffix = suffix,
          ),
        )
        .ok();
      }
    }
    core::fmt::write(&mut w, format_args!("\n")).ok();
  }

  // Single syscall for the entire output.
  let out = w.written();
  let written = unsafe { sys_write(1, out.as_ptr(), out.len()) };
  if written < 0 {
    #[allow(clippy::cast_possible_truncation)]
    return Err(Error::OsError(written as i32));
  }

  #[allow(clippy::cast_sign_loss)]
  if written as usize != out.len() {
    return Err(Error::WriteError);
  }

  Ok(())
}

/// Print version information using direct syscall.
fn print_version() {
  const VERSION: &str = concat!("Microfetch ", env!("CARGO_PKG_VERSION"), "\n");
  unsafe {
    let _ = sys_write(1, VERSION.as_ptr(), VERSION.len());
  }
}

/// Check if --version was passed via argc/argv.
///
/// # Safety
///
/// This function must be called with valid argc and argv from the program entry
/// point.
unsafe fn check_version_flag(argc: i32, argv: *const *const u8) -> bool {
  if argc < 2 {
    return false;
  }
  // SAFETY: argv is a valid array of argc pointers
  let arg1 = unsafe { *argv.add(1) };
  if arg1.is_null() {
    return false;
  }
  // Check if arg1 is "--version"
  let version_flag = b"--version\0";
  for (i, &b) in version_flag.iter().enumerate() {
    // SAFETY: arg1 is a valid C string
    let arg_byte = unsafe { *arg1.add(i) };
    if arg_byte != b {
      return false;
    }
  }
  true
}

/// Main entry point for microfetch - can be called by the binary crate
/// or by other consumers of the library.
///
/// # Arguments
///
/// * `argc` - Argument count from main
/// * `argv` - Argument vector from main
///
/// # Errors
///
/// Returns an error if any system call fails
///
/// # Safety
///
/// argv must be a valid null-terminated array of C strings.
#[cfg_attr(feature = "hotpath", hotpath::main)]
pub unsafe fn run(argc: i32, argv: *const *const u8) -> Result<(), Error> {
  if unsafe { check_version_flag(argc, argv) } {
    print_version();
    return Ok(());
  }

  let utsname = UtsName::uname()?;
  let fields = Fields {
    user_info:      system::get_username_and_hostname(&utsname),
    os_name:        release::get_os_pretty_name()?,
    kernel_version: release::get_system_info(&utsname),
    cpu_name:       cpu::get_cpu_name(),
    cpu_cores:      cpu::get_cpu_cores()?,
    shell:          system::get_shell(),
    desktop:        desktop::get_desktop_info(),
    uptime:         uptime::get_current()?,
    memory_usage:   system::get_memory_usage()?,
    storage:        system::get_root_disk_usage()?,
    colors:         colors::print_dots(),
  };
  print_system_info(&fields)?;

  Ok(())
}
