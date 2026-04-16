#![no_std]

pub mod colors;
pub mod cpu;
pub mod desktop;
pub mod release;
pub mod system;
pub mod uptime;

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
///
/// # Safety guarantee
///
/// Environment variables set by the shell / login process are always valid
/// UTF-8 on any real Linux system. We skip the `from_utf8` validation and use
/// `from_utf8_unchecked`.
#[must_use]
pub fn getenv_str(name: &str) -> Option<&'static str> {
  getenv(name).map(|bytes| unsafe { core::str::from_utf8_unchecked(bytes) })
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

/// Minimal, stack-allocated writer.
pub struct StackWriter<'a> {
  buf: &'a mut [u8],
  pos: usize,
}

impl<'a> StackWriter<'a> {
  #[inline]
  pub const fn new(buf: &'a mut [u8]) -> Self {
    Self { buf, pos: 0 }
  }

  #[inline]
  #[must_use]
  pub fn written(&self) -> &[u8] {
    &self.buf[..self.pos]
  }

  #[inline]
  pub fn push_str(&mut self, s: &str) {
    self.push_bytes(s.as_bytes());
  }

  #[inline]
  pub fn push_bytes(&mut self, bytes: &[u8]) {
    let n = bytes.len().min(self.buf.len() - self.pos);
    self.buf[self.pos..self.pos + n].copy_from_slice(&bytes[..n]);
    self.pos += n;
  }

  #[inline]
  pub fn push_byte(&mut self, b: u8) {
    if self.pos < self.buf.len() {
      self.buf[self.pos] = b;
      self.pos += 1;
    }
  }

  /// Write a [`CStr`]'s bytes (excluding the null terminator).
  #[inline]
  pub fn push_cstr(&mut self, s: &CStr) {
    self.push_bytes(s.to_bytes());
  }

  /// Write a u64 as decimal ASCII.
  pub fn push_u64(&mut self, mut n: u64) {
    if n == 0 {
      self.push_byte(b'0');
      return;
    }
    let mut tmp = [0u8; 20];
    let mut i = 20;
    while n > 0 {
      i -= 1;
      tmp[i] = b'0' + (n % 10) as u8;
      n /= 10;
    }
    self.push_bytes(&tmp[i..]);
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

/// Packed logo rows. `0` separates rows; `1` and `2` select colors.
const LOGO: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/logo.bin"));

/// Write the default two-tone NixOS braille logo for one row, advancing
/// `logo` past the row separator so the next call resumes where this left off.
#[inline(never)]
fn write_logo(w: &mut StackWriter, c: &colors::Colors, logo: &mut &[u8]) {
  let colors = [c.blue, c.cyan];
  let row_data = *logo;

  let mut i = 0;
  while i < row_data.len() {
    match row_data[i] {
      0 => {
        i += 1;
        break;
      },
      1 | 2 => {
        w.push_str(colors[(row_data[i] - 1) as usize]);
        i += 1;
      },
      _ => {
        let chunk_start = i;
        while i < row_data.len() && row_data[i] > 2 {
          i += 1;
        }
        w.push_bytes(&row_data[chunk_start..i]);
      },
    }
  }
  *logo = &row_data[i..];
  w.push_str(c.reset);
}

// Info row labels
struct RowLabel {
  icon:    &'static str,
  key:     &'static str,
  spacing: &'static str,
}

const ROW_LABELS: [Option<RowLabel>; 11] = [
  None, // row 0: user@host
  Some(RowLabel {
    icon:    "\u{F313}  ",
    key:     "System",
    spacing: "       \u{E621} ",
  }),
  Some(RowLabel {
    icon:    "\u{E712}  ",
    key:     "Kernel",
    spacing: "       \u{E621} ",
  }),
  Some(RowLabel {
    icon:    "\u{F2DB}  ",
    key:     "CPU",
    spacing: "          \u{E621} ",
  }),
  Some(RowLabel {
    icon:    "\u{F4BC}  ",
    key:     "Topology",
    spacing: "     \u{E621} ",
  }),
  Some(RowLabel {
    icon:    "\u{E795}  ",
    key:     "Shell",
    spacing: "        \u{E621} ",
  }),
  Some(RowLabel {
    icon:    "\u{F017}  ",
    key:     "Uptime",
    spacing: "       \u{E621} ",
  }),
  Some(RowLabel {
    icon:    "\u{F2D2}  ",
    key:     "Desktop",
    spacing: "      \u{E621} ",
  }),
  Some(RowLabel {
    icon:    "\u{F035B}  ",
    key:     "Memory",
    spacing: "       \u{E621} ",
  }),
  Some(RowLabel {
    icon:    "\u{F194E}  ",
    key:     "Storage (/)",
    spacing: "  \u{E621} ",
  }),
  Some(RowLabel {
    icon:    "\u{E22B}  ",
    key:     "Colors",
    spacing: "       \u{E621} ",
  }),
];

/// Write one row: logo + label + value.
#[inline]
#[allow(clippy::ref_option)]
fn write_row(
  w: &mut StackWriter,
  c: &colors::Colors,
  custom_logo: &str,
  use_custom: bool,
  logo: &mut &[u8],
  label: &Option<RowLabel>,
  write_value: impl FnOnce(&mut StackWriter),
  suffix: &str,
) {
  w.push_str("    ");
  if use_custom {
    w.push_str(c.cyan);
    w.push_str(custom_logo);
    w.push_str(c.reset);
  } else {
    write_logo(w, c, logo);
  }
  w.push_str("  ");
  if let Some(l) = label {
    w.push_str(c.cyan);
    w.push_str(l.icon);
    w.push_str(c.blue);
    w.push_str(l.key);
    w.push_str(c.reset);
    w.push_str(l.spacing);
  }
  write_value(w);
  w.push_str(suffix);
  w.push_byte(b'\n');
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
  let no_color = colors::is_no_color();
  let c = colors::Colors::new(no_color);

  let mut buf = [0u8; 2560];
  let mut w = StackWriter::new(&mut buf);

  // Custom logo is 11 lines from MICROFETCH_LOGO env var, one per info row.
  // Lines beyond 11 are ignored; missing lines render as empty.
  let custom_lines: [&str; 11];
  let use_custom = !CUSTOM_LOGO.is_empty();
  let logo_lines = if use_custom {
    let mut lines_iter = CUSTOM_LOGO.split('\n');
    custom_lines = core::array::from_fn(|_| lines_iter.next().unwrap_or(""));
    &custom_lines
  } else {
    &[""; 11] // unused, we use LOGO pairs below
  };
  let mut logo = LOGO;

  w.push_byte(b'\n');

  macro_rules! row {
    ($idx:expr, $write_value:expr, $suffix:expr) => {
      write_row(
        &mut w,
        &c,
        if use_custom { logo_lines[$idx] } else { "" },
        use_custom,
        &mut logo,
        &ROW_LABELS[$idx],
        $write_value,
        $suffix,
      );
    };
  }

  row!(
    0,
    |w: &mut StackWriter| {
      system::write_username_and_hostname(w, &c, &utsname);
      w.push_str(" ~");
      w.push_str(c.reset);
    },
    ""
  );
  row!(
    1,
    |w: &mut StackWriter| {
      let _ = release::write_os_pretty_name(w);
    },
    ""
  );
  row!(
    2,
    |w: &mut StackWriter| {
      release::write_system_info(w, &utsname);
    },
    ""
  );
  row!(
    3,
    |w: &mut StackWriter| {
      cpu::write_cpu_name(w);
    },
    ""
  );
  row!(
    4,
    |w: &mut StackWriter| {
      let _ = cpu::write_cpu_cores(w);
    },
    ""
  );
  row!(
    5,
    |w: &mut StackWriter| {
      system::write_shell(w);
    },
    ""
  );
  row!(
    6,
    |w: &mut StackWriter| {
      let _ = uptime::write_uptime(w);
    },
    ""
  );
  row!(
    7,
    |w: &mut StackWriter| {
      desktop::write_desktop_info(w);
    },
    ""
  );
  row!(
    8,
    |w: &mut StackWriter| {
      let _ = system::write_memory_usage(w, &c);
    },
    ""
  );
  row!(
    9,
    |w: &mut StackWriter| {
      let _ = system::write_root_disk_usage(w, &c);
    },
    ""
  );
  row!(
    10,
    |w: &mut StackWriter| {
      colors::write_dots(w, &c);
    },
    ""
  );

  w.push_byte(b'\n');

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
