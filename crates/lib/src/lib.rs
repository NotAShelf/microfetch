#![no_std]

// `hotpath::measure` expands to `std` paths.
#[cfg(feature = "hotpath")] extern crate std;

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

use microfetch_asm::{UtsNameBuf, sys_uname, sys_write};

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
  #[must_use = "this only reads errno"]
  pub const fn last_os_error() -> Self {
    // This is a simplified version - in a real implementation,
    // we'd need to get the actual errno from the syscall return
    Self::OsError(0)
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
#[must_use = "this only reads the stored pointer"]
fn get_envp() -> *const *const u8 {
  ENVP.load(Ordering::Acquire)
}

/// Gets an environment variable by name without using std or libc by reading
/// from the environment pointer set by [`init_env`].
#[must_use = "this only reads the environment"]
pub fn getenv(name: &str) -> Option<&'static [u8]> {
  let envp = get_envp();
  if envp.is_null() {
    return None;
  }

  let name_bytes = name.as_bytes();

  // Walk through environment variables
  let mut env_index = 0;
  loop {
    // SAFETY: environ is null-terminated array of pointers
    let entry = unsafe { *envp.add(env_index) };
    if entry.is_null() {
      break;
    }

    // Check if this entry starts with our variable name followed by '='
    let mut matches = true;
    for (name_index, &name_byte) in name_bytes.iter().enumerate() {
      // SAFETY: entry is a valid C string
      let entry_byte = unsafe { *entry.add(name_index) };
      if entry_byte != name_byte {
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
          let value_byte = unsafe { *value_start.add(len) };
          if value_byte == 0 {
            break;
          }
          len += 1;
        }
        // SAFETY: We calculated the exact length
        return Some(unsafe { core::slice::from_raw_parts(value_start, len) });
      }
    }

    env_index += 1;
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
#[must_use = "this only reads the environment"]
pub fn getenv_str(name: &str) -> Option<&'static str> {
  getenv(name).map(|bytes| unsafe { core::str::from_utf8_unchecked(bytes) })
}

/// Checks if an environment variable exists (regardless of its value).
#[must_use = "this only reads the environment"]
pub fn env_exists(name: &str) -> bool {
  getenv(name).is_some()
}

/// Wrapper for `utsname` with safe accessor methods.
pub struct UtsName(UtsNameBuf);

impl UtsName {
  /// Calls `uname(2)` syscall and returns a `UtsName` wrapper.
  ///
  /// # Errors
  ///
  /// Returns an error if the `uname` syscall fails.
  pub fn uname() -> Result<Self, Error> {
    let mut uts = MaybeUninit::uninit();
    if unsafe { sys_uname(uts.as_mut_ptr()) } != 0_i32 {
      return Err(Error::last_os_error());
    }

    Ok(Self(unsafe { uts.assume_init() }))
  }

  #[must_use = "this only borrows a field"]
  pub const fn nodename(&self) -> &CStr {
    unsafe { CStr::from_ptr(self.0.nodename.as_ptr().cast()) }
  }

  #[must_use = "this only borrows a field"]
  pub const fn sysname(&self) -> &CStr {
    unsafe { CStr::from_ptr(self.0.sysname.as_ptr().cast()) }
  }

  #[must_use = "this only borrows a field"]
  pub const fn release(&self) -> &CStr {
    unsafe { CStr::from_ptr(self.0.release.as_ptr().cast()) }
  }

  #[must_use = "this only borrows a field"]
  pub const fn machine(&self) -> &CStr {
    unsafe { CStr::from_ptr(self.0.machine.as_ptr().cast()) }
  }
}

/// Minimal, stack-allocated writer.
pub struct StackWriter<'buf> {
  buf: &'buf mut [MaybeUninit<u8>],
  pos: usize,
}

impl<'buf> StackWriter<'buf> {
  #[inline]
  pub const fn new(buf: &'buf mut [MaybeUninit<u8>]) -> Self {
    Self { buf, pos: 0 }
  }

  #[inline]
  #[must_use = "this only borrows the written bytes"]
  pub const fn written(&self) -> &[u8] {
    unsafe { core::slice::from_raw_parts(self.buf.as_ptr().cast(), self.pos) }
  }

  #[inline]
  pub fn push_str(&mut self, text: &str) {
    self.push_bytes(text.as_bytes());
  }

  #[inline(never)]
  pub fn push_bytes(&mut self, bytes: &[u8]) {
    let len = bytes.len().min(self.buf.len() - self.pos);
    unsafe {
      core::ptr::copy_nonoverlapping(
        bytes.as_ptr(),
        self.buf.as_mut_ptr().add(self.pos).cast(),
        len,
      );
    };
    self.pos += len;
  }

  #[inline]
  pub const fn push_byte(&mut self, byte: u8) {
    if self.pos < self.buf.len() {
      self.buf[self.pos].write(byte);
      self.pos += 1;
    }
  }

  /// Write a [`CStr`]'s bytes (excluding the null terminator).
  #[inline]
  pub fn push_cstr(&mut self, cstr: &CStr) {
    self.push_bytes(cstr.to_bytes());
  }

  /// Write a u64 as decimal ASCII.
  pub fn push_u64(&mut self, mut value: u64) {
    if value == 0 {
      self.push_byte(b'0');
      return;
    }
    let mut digits = [0_u8; 20];
    let mut start = 20;
    while value > 0 {
      start -= 1;
      digits[start] = b'0' + u8::try_from(value % 10).unwrap_or(0);
      value /= 10;
    }
    self.push_bytes(&digits[start..]);
  }
}

/// Custom logo art embedded at compile time via the `MICROFETCH_LOGO`
/// environment variable. Set it to 11 newline-separated lines of ASCII/Unicode
/// art when building to replace the default NixOS logo:
///
/// ```sh
/// MICROFETCH_LOGO="$(cat my_logo.txt)" cargo build --release
/// ```
///
/// Each line maps to one info row. When unset, the built-in two-tone NixOS
/// logo is used.
const CUSTOM_LOGO: &str = match option_env!("MICROFETCH_LOGO") {
  Some(logo) => logo,
  None => "",
};

/// Packed logo rows. `0` separates rows; `1` and `2` select colors.
const LOGO: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/logo.bin"));

/// Write one row from a packed color-marked byte stream.
#[inline(never)]
fn write_styled_row(
  writer: &mut StackWriter,
  colors: &colors::Colors,
  stream: &mut &[u8],
) {
  let palette = [colors.blue, colors.cyan, colors.reset];
  let data = *stream;
  let row_end = data
    .iter()
    .position(|&byte| byte == 0)
    .unwrap_or(data.len());
  let row_data = &data[..row_end];
  *stream = data.get(row_end + 1..).unwrap_or(&[]);

  let mut pos = 0;
  while pos < row_data.len() {
    if let marker @ 1..=3 = row_data[pos] {
      writer.push_str(palette[usize::from(marker - 1)]);
      pos += 1;
    } else {
      let chunk_start = pos;
      while pos < row_data.len() && row_data[pos] > 3 {
        pos += 1;
      }
      writer.push_bytes(&row_data[chunk_start..pos]);
    }
  }
}

const ROW_COUNT: usize = 11;
const ROW_LABELS: &[u8] = concat!(
  "\x02\u{F313}  \x01System\x03       \u{E621} \0",
  "\x02\u{E712}  \x01Kernel\x03       \u{E621} \0",
  "\x02\u{F2DB}  \x01CPU\x03          \u{E621} \0",
  "\x02\u{F4BC}  \x01Topology\x03     \u{E621} \0",
  "\x02\u{E795}  \x01Shell\x03        \u{E621} \0",
  "\x02\u{F017}  \x01Uptime\x03       \u{E621} \0",
  "\x02\u{F2D2}  \x01Desktop\x03      \u{E621} \0",
  "\x02\u{F035B}  \x01Memory\x03       \u{E621} \0",
  "\x02\u{F194E}  \x01Storage (/)\x03  \u{E621} \0",
  "\x02\u{E22B}  \x01Colors\x03       \u{E621} \0",
)
.as_bytes();

/// Write one row: logo + label + value.
#[inline(never)]
fn write_row(
  writer: &mut StackWriter,
  colors: &colors::Colors,
  row: usize,
  custom_logo: Option<&str>,
  logo: &mut &[u8],
  labels: &mut &[u8],
  utsname: &UtsName,
) {
  writer.push_str("    ");
  if let Some(logo_text) = custom_logo {
    writer.push_str(colors.cyan);
    writer.push_str(logo_text);
  } else {
    write_styled_row(writer, colors, logo);
  }
  writer.push_str(colors.reset);
  writer.push_str("  ");
  if row != 0 {
    write_styled_row(writer, colors, labels);
  }
  match row {
    0 => {
      system::write_username_and_hostname(writer, colors, utsname);
      writer.push_str(" ~");
      writer.push_str(colors.reset);
    },
    1 => release::write_os_pretty_name(writer),
    2 => release::write_system_info(writer, utsname),
    3 => cpu::write_cpu_name(writer),
    4 => cpu::write_cpu_cores(writer),
    5 => system::write_shell(writer),
    6 => uptime::write(writer),
    7 => desktop::write_desktop_info(writer),
    8 => system::write_memory_usage(writer, colors),
    9 => system::write_root_disk_usage(writer, colors),
    _ => colors::write_dots(writer, colors),
  }
  writer.push_byte(b'\n');
}

/// Print version information using direct syscall.
fn print_version() {
  const VERSION: &str = concat!("Microfetch ", env!("CARGO_PKG_VERSION"), "\n");
  unsafe {
    _ = sys_write(1, VERSION.as_ptr(), VERSION.len());
  }
}

/// Check if --version was passed via argc/argv.
///
/// # Safety
///
/// This function must be called with valid argc and argv from the program entry
/// point.
unsafe fn check_version_flag(argc: i32, argv: *const *const u8) -> bool {
  if argc < 2_i32 {
    return false;
  }
  // SAFETY: argv is a valid array of argc pointers
  let arg1 = unsafe { *argv.add(1) };
  if arg1.is_null() {
    return false;
  }
  // Check if arg1 is "--version"
  let version_flag = b"--version\0";
  for (flag_index, &flag_byte) in version_flag.iter().enumerate() {
    // SAFETY: arg1 is a valid C string
    let arg_byte = unsafe { *arg1.add(flag_index) };
    if arg_byte != flag_byte {
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
/// Returns an error if any system call fails.
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
  let colors = colors::Colors::new(no_color);

  let mut buf = [MaybeUninit::uninit(); 2560];
  let mut writer = StackWriter::new(&mut buf);

  // Custom logo is 11 lines from MICROFETCH_LOGO env var, one per info row.
  // Lines beyond 11 are ignored; missing lines render as empty.
  let custom_lines: [&str; ROW_COUNT];
  let use_custom = !CUSTOM_LOGO.is_empty();
  let logo_lines = if use_custom {
    let mut lines_iter = CUSTOM_LOGO.split('\n');
    custom_lines = core::array::from_fn(|_| lines_iter.next().unwrap_or(""));
    &custom_lines
  } else {
    &[""; ROW_COUNT] // unused, we use LOGO pairs below
  };
  let mut logo = LOGO;
  let mut labels = ROW_LABELS;

  writer.push_byte(b'\n');

  for (row, &line) in logo_lines.iter().enumerate() {
    write_row(
      &mut writer,
      &colors,
      row,
      use_custom.then_some(line),
      &mut logo,
      &mut labels,
      &utsname,
    );
  }

  writer.push_byte(b'\n');

  // Single syscall for the entire output.
  let out = writer.written();
  let written = unsafe { sys_write(1, out.as_ptr(), out.len()) };
  let Ok(count) = usize::try_from(written) else {
    return Err(
      i32::try_from(written).map_or(Error::WriteError, Error::OsError),
    );
  };

  if count != out.len() {
    return Err(Error::WriteError);
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn stack_writer_truncates_at_capacity() {
    let mut buf = [MaybeUninit::uninit(); 4];
    let mut writer = StackWriter::new(&mut buf);
    writer.push_bytes(b"abc");
    writer.push_byte(b'd');
    writer.push_str("overflow");
    assert_eq!(writer.written(), b"abcd");
  }

  #[test]
  fn styled_rows_advance_the_stream() {
    let mut buf = [MaybeUninit::uninit(); 3];
    let mut writer = StackWriter::new(&mut buf);
    let mut rows = &b"\x02a\x01b\x03c\0tail"[..];
    write_styled_row(&mut writer, &colors::Colors::new(true), &mut rows);
    assert_eq!(writer.written(), b"abc");
    assert_eq!(rows, b"tail");
  }
}
