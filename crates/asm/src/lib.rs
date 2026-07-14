//! Incredibly fast syscall wrappers for using inline assembly. Serves the
//! purposes of completely bypassing Rust's standard library in favor of
//! handwritten Assembly. Is this a good idea? No. Is it fast? Yeah, but only
//! marginally. Either way it serves a purpose and I will NOT accept criticism.
//! What do you mean I wasted two whole hours to make the program only 100µs
//! faster?
//!
//! Supports `x86_64`, `aarch64`, `riscv64`, `loongarch64`, `s390x`,
//! `powerpc64`, `arm` (armv7), `riscv32`, `sparc64`, `mips64`, `x86` (i686),
//! `powerpc` (ppc32), `sparc` (sparc32), and `mips` (O32) architectures.

#![no_std]
#![cfg_attr(
  any(
    target_arch = "sparc64",
    target_arch = "sparc",
    target_arch = "mips64",
    target_arch = "mips"
  ),
  feature(asm_experimental_arch)
)]

#[cfg(not(target_os = "macos"))] use core::ffi::c_void;

// Per-arch syscall implementations live in their own module files.
// macOS is matched first; there `target_arch` is `aarch64`, but the
// implementation is libSystem-backed rather than raw `svc` traps.
core::cfg_select! {
  all(target_os = "macos", target_arch = "aarch64") => { #[path = "darwin.rs"  ] mod arch; }
  target_os = "macos"         => { compile_error!("microfetch on macOS supports aarch64 (Apple Silicon) only"); }
  target_arch = "x86_64"      => { #[path = "x86_64.rs"     ] mod arch;        }
  target_arch = "aarch64"     => { #[path = "aarch64.rs"    ] mod arch;        }
  target_arch = "riscv64"     => { #[path = "riscv64.rs"    ] mod arch;        }
  target_arch = "loongarch64" => { #[path = "loongarch64.rs"] mod arch;        }
  target_arch = "s390x"       => { #[path = "s390x.rs"      ] mod arch;        }
  target_arch = "powerpc64"   => { #[path = "powerpc64.rs"  ] mod arch;        }
  target_arch = "arm"         => { #[path = "arm.rs"        ] mod arch;        }
  target_arch = "riscv32"     => { #[path = "riscv32.rs"    ] mod arch;        }
  target_arch = "sparc64"     => { #[path = "sparc64.rs"    ] mod arch;        }
  target_arch = "mips64"      => { #[path = "mips64.rs"     ] mod arch;        }
  target_arch = "x86"         => { #[path = "x86.rs"        ] mod arch;        }
  target_arch = "powerpc"     => { #[path = "powerpc.rs"    ] mod arch;        }
  target_arch = "sparc"       => { #[path = "sparc.rs"      ] mod arch;        }
  target_arch = "mips"        => { #[path = "mips.rs"       ] mod arch;        }
  _                           => { compile_error!("Unsupported architecture"); }
}

/// macOS-only helpers backed by `sysctl`/Mach (see `darwin.rs`).
#[cfg(target_os = "macos")]
pub use arch::{
  macos_meminfo,
  macos_sysctl_str,
  macos_sysctl_u32,
  macos_uptime_secs,
};

/// Copies `n` bytes from `src` to `dest`.
///
/// # Safety
///
/// `dest` and `src` must be valid pointers to non-overlapping regions of
/// memory of at least `n` bytes.
// On macOS these symbols are supplied by libSystem; defining our own would
// clash at link time, so the freestanding implementations are Linux-only.
#[cfg(not(target_os = "macos"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(
  dest: *mut c_void,
  src: *const c_void,
  n: usize,
) -> *mut c_void {
  let dest_bytes = dest.cast::<u8>();
  let src_bytes = src.cast::<u8>();
  for i in 0..n {
    unsafe {
      *dest_bytes.add(i) = *src_bytes.add(i);
    }
  }
  dest
}

/// Fills memory region with a byte value.
///
/// # Safety
///
/// `s` must be a valid pointer to memory of at least `n` bytes.
/// The value in `c` is treated as unsigned (lower 8 bits used).
#[cfg(not(target_os = "macos"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(
  s: *mut c_void,
  c: i32,
  n: usize,
) -> *mut c_void {
  let bytes = s.cast::<u8>();
  let value = c.to_le_bytes()[0];
  for i in 0..n {
    unsafe {
      *bytes.add(i) = value;
    }
  }
  s
}

/// Compares two byte sequences.
///
/// # Safety
///
/// `s1` and `s2` must be valid pointers to memory of at least `n` bytes.
#[cfg(not(target_os = "macos"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bcmp(
  s1: *const c_void,
  s2: *const c_void,
  n: usize,
) -> i32 {
  let bytes1 = s1.cast::<u8>();
  let bytes2 = s2.cast::<u8>();
  for i in 0..n {
    let a = unsafe { *bytes1.add(i) };
    let b = unsafe { *bytes2.add(i) };
    if a != b {
      return i32::from(a) - i32::from(b);
    }
  }
  0
}

/// Compares two byte sequences.
///
/// # Safety
///
/// `s1` and `s2` must be valid pointers to memory of at least `n` bytes.
#[cfg(not(target_os = "macos"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcmp(
  s1: *const c_void,
  s2: *const c_void,
  n: usize,
) -> i32 {
  unsafe { bcmp(s1, s2, n) }
}

/// Calculates the length of a null-terminated string.
///
/// # Safety
///
/// `s` must be a valid pointer to a null-terminated string.
#[cfg(not(target_os = "macos"))]
#[unsafe(no_mangle)]
pub const unsafe extern "C" fn strlen(s: *const i8) -> usize {
  let mut len = 0;
  while unsafe { *s.add(len) } != 0 {
    len += 1;
  }
  len
}

/// Function pointer type for the main application entry point.
/// The function receives argc and argv and should return an exit code.
// On macOS the C runtime calls `main` directly, so the custom `_start` /
// `entry_rust` path below is Linux-only.
#[cfg(all(not(test), not(target_os = "macos")))]
pub type MainFn = unsafe extern "C" fn(i32, *const *const u8) -> i32;

#[cfg(all(not(test), not(target_os = "macos")))]
static mut MAIN_FN: Option<MainFn> = None;

/// Register the main function to be called from the entry point.
/// This must be called before the program starts (e.g., in a constructor).
#[cfg(all(not(test), not(target_os = "macos")))]
pub fn register_main(main_fn: MainFn) {
  unsafe {
    MAIN_FN = Some(main_fn);
  }
}

/// Rust entry point called from `_start` assembly.
///
/// The `stack` pointer points to:
/// `[rsp]`     = argc
/// `[rsp+8]`   = argv[0]
/// etc.
///
/// # Safety
///
/// The `stack` pointer must point to valid stack memory set up by the kernel
/// AND the binary must define a `main` function with the following signature:
///
/// ```rust,ignore
/// unsafe extern "C" fn main(argc: i32, argv: *const *const u8) -> i32`
/// ```
#[cfg(all(not(test), not(target_os = "macos")))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn entry_rust(stack: *const usize) -> i32 {
  // Read argc and argv from stack
  let argc = unsafe { *stack };
  let argv = unsafe { stack.add(1).cast::<*const u8>() };

  // SAFETY: argc is unlikely to exceed i32::MAX on real systems
  let argc_i32 = i32::try_from(argc).unwrap_or(i32::MAX);

  // Call the main function (defined by the binary crate)
  unsafe { main(argc_i32, argv) }
}

// External main function that must be defined by the binary using this crate.
// Signature: `unsafe extern "C" fn main(argc: i32, argv: *const *const u8) ->
// i32`
#[cfg(all(not(test), not(target_os = "macos")))]
unsafe extern "C" {
  fn main(argc: i32, argv: *const *const u8) -> i32;
}

/// Direct syscall to open a file
///
/// # Returns
///
/// File descriptor or -1 on error
///
/// # Safety
///
/// The caller must ensure:
///
/// - `path` points to a valid null-terminated C string
/// - The pointer remains valid for the duration of the syscall
#[inline]
#[must_use]
pub unsafe fn sys_open(path: *const u8, flags: i32) -> i32 {
  unsafe { arch::sys_open(path, flags) }
}

/// Direct syscall to read from a file descriptor
///
/// # Returns
///
/// Number of bytes read or -1 on error
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `buf` points to a valid writable buffer of at least `count` bytes
/// - `fd` is a valid open file descriptor
#[inline]
pub unsafe fn sys_read(fd: i32, buf: *mut u8, count: usize) -> isize {
  unsafe { arch::sys_read(fd, buf, count) }
}

/// Direct syscall to write to a file descriptor
///
/// # Returns
///
/// Number of bytes written or -1 on error
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `buf` points to a valid readable buffer of at least `count` bytes
/// - `fd` is a valid open file descriptor
#[inline]
#[must_use]
pub unsafe fn sys_write(fd: i32, buf: *const u8, count: usize) -> isize {
  unsafe { arch::sys_write(fd, buf, count) }
}

/// Direct syscall to close a file descriptor
///
/// # Safety
///
/// The caller must ensure that `fd` is a valid open file descriptor
#[inline]
#[must_use]
pub unsafe fn sys_close(fd: i32) -> i32 {
  unsafe { arch::sys_close(fd) }
}

/// Raw buffer for the `uname(2)` syscall.
///
/// Linux ABI hasfive fields of `[i8; 65]`: sysname, nodename, release, version,
/// machine. The `domainname` field (GNU extension, `[i8; 65]`) follows but is
/// not used, nor any useful to us here.
#[repr(C)]
#[allow(dead_code)]
#[cfg(not(target_os = "macos"))]
pub struct UtsNameBuf {
  pub sysname:    [i8; 65],
  pub nodename:   [i8; 65],
  pub release:    [i8; 65],
  pub version:    [i8; 65],
  pub machine:    [i8; 65],
  pub domainname: [i8; 65], // GNU extension, included for correct struct size
}

/// macOS `struct utsname`: five `char[_SYS_NAMELEN]` fields (`_SYS_NAMELEN`
/// is 256) and no `domainname`. Field names match the Linux layout so the
/// `UtsName` accessors in `microfetch-lib` work unchanged.
#[repr(C)]
#[cfg(target_os = "macos")]
pub struct UtsNameBuf {
  pub sysname:  [i8; 256],
  pub nodename: [i8; 256],
  pub release:  [i8; 256],
  pub version:  [i8; 256],
  pub machine:  [i8; 256],
}

/// Direct `uname(2)` syscall
///
/// # Returns
///
/// 0 on success, negative on error
///
/// # Safety
///
/// The caller must ensure that `buf` points to a valid `UtsNameBuf`.
#[inline]
#[allow(dead_code)]
pub unsafe fn sys_uname(buf: *mut UtsNameBuf) -> i32 {
  unsafe { arch::sys_uname(buf) }
}

/// Raw buffer for the `statfs(2)` syscall.
///
/// Linux ABI (`x86_64` and `aarch64`): the fields we use are at the same
/// offsets on both architectures. Only the fields needed for disk usage are
/// declared; the remainder of the 120-byte struct is covered by `_pad`.
#[repr(C)]
#[cfg(not(any(
  target_os = "macos",
  target_arch = "s390x",
  target_arch = "arm",
  target_arch = "riscv32",
  target_arch = "x86",
  target_arch = "powerpc",
  target_arch = "sparc",
  target_arch = "mips64",
  target_arch = "mips"
)))]
pub struct StatfsBuf {
  pub f_type:    i64,
  pub f_bsize:   i64,
  pub f_blocks:  u64,
  pub f_bfree:   u64,
  pub f_bavail:  u64,
  pub f_files:   u64,
  pub f_ffree:   u64,
  pub f_fsid:    [i32; 2],
  pub f_namelen: i64,
  pub f_frsize:  i64,
  pub f_flags:   i64,

  #[allow(clippy::pub_underscore_fields, reason = "This is not a public API")]
  pub _pad: [i64; 4],
}

/// macOS `struct statfs` (the 64-bit/`INODE64` ABI, which is the only one on
/// arm64). Only `f_bsize`, `f_blocks`, and `f_bavail` are read; the rest is
/// declared to give the struct its correct size and field offsets.
#[repr(C)]
#[cfg(target_os = "macos")]
pub struct StatfsBuf {
  pub f_bsize:       u32,
  pub f_iosize:      i32,
  pub f_blocks:      u64,
  pub f_bfree:       u64,
  pub f_bavail:      u64,
  pub f_files:       u64,
  pub f_ffree:       u64,
  pub f_fsid:        [i32; 2],
  pub f_owner:       u32,
  pub f_type:        u32,
  pub f_flags:       u32,
  pub f_fssubtype:   u32,
  pub f_fstypename:  [i8; 16],
  pub f_mntonname:   [i8; 1024],
  pub f_mntfromname: [i8; 1024],
  pub f_flags_ext:   u32,
  pub f_reserved:    [u32; 7],
}

/// on s390x `f_type` and `f_bsize` are 32-bit.
#[repr(C)]
#[cfg(target_arch = "s390x")]
pub struct StatfsBuf {
  pub f_type:    u32,
  pub f_bsize:   u32,
  pub f_blocks:  u64,
  pub f_bfree:   u64,
  pub f_bavail:  u64,
  pub f_files:   u64,
  pub f_ffree:   u64,
  pub f_fsid:    [i32; 2],
  pub f_namelen: u32,
  pub f_frsize:  u32,
  pub f_flags:   u32,

  #[allow(clippy::pub_underscore_fields, reason = "This is not a public API")]
  pub _pad: [u32; 5],
}

/// on armv7 `statfs64(2)` has 32-bit word fields; see
/// https://github.com/torvalds/linux/blob/v6.19/include/uapi/asm-generic/statfs.h
#[repr(C)]
#[cfg(any(
  target_arch = "arm",
  target_arch = "riscv32",
  target_arch = "x86",
  target_arch = "powerpc",
  target_arch = "sparc"
))]
pub struct StatfsBuf {
  pub f_type:    u32,
  pub f_bsize:   u32,
  pub f_blocks:  u64,
  pub f_bfree:   u64,
  pub f_bavail:  u64,
  pub f_files:   u64,
  pub f_ffree:   u64,
  pub f_fsid:    [i32; 2],
  pub f_namelen: u32,
  pub f_frsize:  u32,
  pub f_flags:   u32,

  #[allow(clippy::pub_underscore_fields, reason = "This is not a public API")]
  pub _pad: [u32; 4],
}

/// mips (O32) uses `compat_statfs64`, same reordering with 32-bit words; see
/// https://github.com/torvalds/linux/blob/v6.19/arch/mips/include/uapi/asm/statfs.h
#[repr(C)]
#[cfg(target_arch = "mips")]
pub struct StatfsBuf {
  pub f_type:    u32,
  pub f_bsize:   u32,
  pub f_frsize:  u32,
  _pad:          u32,
  pub f_blocks:  u64,
  pub f_bfree:   u64,
  pub f_files:   u64,
  pub f_ffree:   u64,
  pub f_bavail:  u64,
  pub f_fsid:    [i32; 2],
  pub f_namelen: u32,
  pub f_flags:   u32,

  #[allow(clippy::pub_underscore_fields, reason = "This is not a public API")]
  pub _pad2: [u32; 5],
}

/// mips reorders fields; see
/// https://github.com/torvalds/linux/blob/v6.19/arch/mips/include/uapi/asm/statfs.h
#[repr(C)]
#[cfg(target_arch = "mips64")]
pub struct StatfsBuf {
  pub f_type:    i64,
  pub f_bsize:   i64,
  pub f_frsize:  i64,
  pub f_blocks:  u64,
  pub f_bfree:   u64,
  pub f_files:   u64,
  pub f_ffree:   u64,
  pub f_bavail:  u64,
  pub f_fsid:    [i32; 2],
  pub f_namelen: i64,
  pub f_flags:   i64,

  #[allow(clippy::pub_underscore_fields, reason = "This is not a public API")]
  pub _pad: [i64; 5],
}

/// Direct `statfs(2)` syscall
///
/// # Returns
///
/// 0 on success, negative errno on error
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `path` points to a valid null-terminated string
/// - `buf` points to a valid `StatfsBuf`
#[inline]
pub unsafe fn sys_statfs(path: *const u8, buf: *mut StatfsBuf) -> i32 {
  unsafe { arch::sys_statfs(path, buf) }
}

/// Read entire file using direct syscalls. This avoids libc overhead and can be
/// significantly faster for small files.
///
/// # Errors
///
/// Returns an error if the file cannot be opened or read. The error value is
/// the negated errno.
#[inline]
pub fn read_file_fast(path: &str, buffer: &mut [u8]) -> Result<usize, i32> {
  const O_RDONLY: i32 = 0;

  // We use stack-allocated buffer for null-terminated path. The maximum
  // is 256 bytes.
  let path_bytes = path.as_bytes();
  if path_bytes.len() >= 256 {
    return Err(-22); // EINVAL
  }

  let mut path_buf = [0u8; 256];
  path_buf[..path_bytes.len()].copy_from_slice(path_bytes);
  // XXX: Already zero-terminated since array is initialized to zeros

  unsafe {
    let fd = sys_open(path_buf.as_ptr(), O_RDONLY);
    if fd < 0 {
      return Err(fd);
    }

    let bytes_read = sys_read(fd, buffer.as_mut_ptr(), buffer.len());
    let _ = sys_close(fd);

    if bytes_read < 0 {
      #[allow(clippy::cast_possible_truncation)]
      return Err(bytes_read as i32);
    }

    #[allow(clippy::cast_sign_loss)]
    {
      Ok(bytes_read as usize)
    }
  }
}

/// Raw buffer for the `sysinfo(2)` syscall.
///
/// In the Linux ABI `uptime` is a `long` at offset 0. The remaining fields are
/// not needed, but are declared to give the struct its correct size (112 bytes
/// on 64-bit Linux).
///
/// The layout matches the kernel's `struct sysinfo` *exactly*:
/// `mem_unit` ends at offset 108, then 4 bytes of implicit padding to 112.
#[repr(C)]
#[cfg(not(any(
  target_arch = "arm",
  target_arch = "riscv32",
  target_arch = "x86",
  target_arch = "powerpc",
  target_arch = "sparc",
  target_arch = "mips"
)))]
pub struct SysInfo {
  pub uptime:    i64,
  pub loads:     [u64; 3],
  pub totalram:  u64,
  pub freeram:   u64,
  pub sharedram: u64,
  pub bufferram: u64,
  pub totalswap: u64,
  pub freeswap:  u64,
  pub procs:     u16,
  _pad:          u16,
  _pad2:         u32, /* alignment padding to reach 8-byte boundary for
                       * totalhigh */
  pub totalhigh: u64,
  pub freehigh:  u64,
  pub mem_unit:  u32,
  // 4 bytes implicit trailing padding to reach 112 bytes total; no field
  // needed
}

/// on armv7 `__kernel_long_t` is 4 bytes; see
/// https://github.com/torvalds/linux/blob/v6.19/include/uapi/linux/sysinfo.h
#[repr(C)]
#[cfg(any(
  target_arch = "arm",
  target_arch = "riscv32",
  target_arch = "x86",
  target_arch = "powerpc",
  target_arch = "sparc",
  target_arch = "mips"
))]
pub struct SysInfo {
  pub uptime:    i32,
  pub loads:     [u32; 3],
  pub totalram:  u32,
  pub freeram:   u32,
  pub sharedram: u32,
  pub bufferram: u32,
  pub totalswap: u32,
  pub freeswap:  u32,
  pub procs:     u16,
  _pad:          u16,
  pub totalhigh: u32,
  pub freehigh:  u32,
  pub mem_unit:  u32,
  #[allow(clippy::pub_underscore_fields, reason = "This is not a public API")]
  pub _f:        [u8; 8],
}

/// Direct `sysinfo(2)` syscall
///
/// # Returns
///
/// 0 on success, negative errno on error
///
/// # Safety
///
/// The caller must ensure that `info` points to a valid `SysInfo` buffer.
#[inline]
pub unsafe fn sys_sysinfo(info: *mut SysInfo) -> i64 {
  unsafe { arch::sys_sysinfo(info) }
}

/// Direct `sched_getaffinity(2)` syscall
///
/// # Returns
///
/// On success, the number of bytes written to the mask buffer (always a
/// multiple of `sizeof(long)`). On error, a negative errno.
///
/// # Safety
///
/// The caller must ensure that `mask` points to a buffer of at least
/// `mask_size` bytes.
#[inline]
pub unsafe fn sys_sched_getaffinity(
  pid: i32,
  mask_size: usize,
  mask: *mut u8,
) -> i32 {
  unsafe { arch::sys_sched_getaffinity(pid, mask_size, mask) }
}

/// Direct syscall to exit the process
///
/// # Safety
///
/// This syscall never returns. The process will terminate immediately.
#[inline]
pub unsafe fn sys_exit(code: i32) -> ! {
  unsafe { arch::sys_exit(code) }
}

// The freestanding memory symbols exist everywhere but macOS, where
// libSystem supplies them instead.
#[cfg(all(test, not(target_os = "macos")))]
mod tests {
  use super::*;

  #[test]
  fn runtime_memory_symbols_match_c_semantics() {
    let src = [1u8, 2, 3, 4];
    let mut dest = [0u8; 4];

    unsafe {
      memcpy(dest.as_mut_ptr().cast(), src.as_ptr().cast(), src.len());
    }
    assert_eq!(dest, src);
    assert_eq!(
      unsafe { memcmp(dest.as_ptr().cast(), src.as_ptr().cast(), 4) },
      0
    );

    unsafe {
      memset(dest.as_mut_ptr().cast(), -1, dest.len());
    }
    assert_eq!(dest, [u8::MAX; 4]);
    assert_ne!(
      unsafe { bcmp(dest.as_ptr().cast(), src.as_ptr().cast(), 4) },
      0
    );
  }

  #[test]
  fn runtime_strlen_uses_c_char_pointer() {
    assert_eq!(unsafe { strlen(c"microfetch".as_ptr().cast()) }, 10);
  }
}
