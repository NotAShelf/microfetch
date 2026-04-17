//! macOS does not support a stable raw-syscall ABI the way Linux does, cannot
//! be linked statically, and exposes none of the `/proc`, `/sys`, or
//! `/etc/os-release` interfaces the Linux path relies on. So instead of
//! handwritten `svc #0` traps, this module calls into `libSystem` (which is
//! always linked on macOS) via plain FFI:
//!
//!   * file/IO + exit  -> `open`/`read`/`write`/`close`/`_exit`
//!   * `uname(3)`       -> kernel name/release/machine
//!   * `statfs(2)`      -> root filesystem usage
//!   * `sysctl`/`sysctlbyname` -> CPU model, core counts, RAM size, OS version
//!   * `host_statistics64` (Mach) -> live VM page statistics for memory usage
//!
//! You could argue that linking libSystem is no different than linking libc,
//! and while yes you would be correct, I'm not doing this without libSystem.
//! Talk is cheap, send patches.
use core::{
  ffi::{c_char, c_int, c_void},
  mem::size_of,
  ptr,
};

use super::{StatfsBuf, SysInfo, UtsNameBuf};

#[link(name = "System", kind = "dylib")]
unsafe extern "C" {
  fn open(path: *const c_char, flags: c_int, ...) -> c_int;
  fn read(fd: c_int, buf: *mut c_void, count: usize) -> isize;
  fn write(fd: c_int, buf: *const c_void, count: usize) -> isize;
  fn close(fd: c_int) -> c_int;
  fn _exit(code: c_int) -> !;

  fn uname(buf: *mut UtsNameBuf) -> c_int;
  fn statfs(path: *const c_char, buf: *mut StatfsBuf) -> c_int;

  fn sysctl(
    name: *const c_int,
    namelen: u32,
    oldp: *mut c_void,
    oldlenp: *mut usize,
    newp: *const c_void,
    newlen: usize,
  ) -> c_int;
  fn sysctlbyname(
    name: *const c_char,
    oldp: *mut c_void,
    oldlenp: *mut usize,
    newp: *const c_void,
    newlen: usize,
  ) -> c_int;
  fn gettimeofday(tp: *mut Timeval, tzp: *mut c_void) -> c_int;

  // Mach host interfaces (live in libsystem_kernel, part of libSystem).
  fn mach_host_self() -> u32; // mach_port_t
  fn host_statistics64(
    host: u32,
    flavor: c_int,
    info: *mut i32, // host_info64_t == integer_t*
    count: *mut u32,
  ) -> c_int;
}

pub(super) unsafe fn sys_open(path: *const u8, flags: i32) -> i32 {
  unsafe { open(path.cast::<c_char>(), flags) }
}

pub(super) unsafe fn sys_read(fd: i32, buf: *mut u8, count: usize) -> isize {
  unsafe { read(fd, buf.cast::<c_void>(), count) }
}

pub(super) unsafe fn sys_write(fd: i32, buf: *const u8, count: usize) -> isize {
  unsafe { write(fd, buf.cast::<c_void>(), count) }
}

pub(super) unsafe fn sys_close(fd: i32) -> i32 {
  unsafe { close(fd) }
}

pub(super) unsafe fn sys_uname(buf: *mut UtsNameBuf) -> i32 {
  unsafe { uname(buf) }
}

pub(super) unsafe fn sys_statfs(path: *const u8, buf: *mut StatfsBuf) -> i32 {
  unsafe { statfs(path.cast::<c_char>(), buf) }
}

/// macOS has no `sysinfo(2)`; uptime comes from [`macos_uptime_secs`] instead.
/// This stub exists only so the generic wrapper in `lib.rs` keeps compiling.
pub(super) unsafe fn sys_sysinfo(_info: *mut SysInfo) -> i64 {
  -1
}

/// macOS has no `sched_getaffinity(2)`; core counts come from `sysctl`
/// (`hw.logicalcpu` / `hw.physicalcpu`). This stub keeps the generic wrapper
/// compiling.
pub(super) unsafe fn sys_sched_getaffinity(
  _pid: i32,
  _mask_size: usize,
  _mask: *mut u8,
) -> i32 {
  -1
}

pub(super) unsafe fn sys_exit(code: i32) -> ! {
  unsafe { _exit(code) }
}

/// Read a string-valued sysctl by name into `out`. `name` must be a
/// NUL-terminated byte string (e.g. `b"machdep.cpu.brand_string\0"`).
///
/// Returns the number of bytes written, excluding the trailing NUL.
#[must_use]
pub fn macos_sysctl_str(name: &[u8], out: &mut [u8]) -> Option<usize> {
  let mut len = out.len();
  let ret = unsafe {
    sysctlbyname(
      name.as_ptr().cast::<c_char>(),
      out.as_mut_ptr().cast::<c_void>(),
      &mut len,
      ptr::null(),
      0,
    )
  };
  if ret != 0 {
    return None;
  }
  // `len` includes the terminating NUL on success.
  Some(len.saturating_sub(1))
}

/// Read a 32-bit integer sysctl by name (e.g. `b"hw.logicalcpu\0"`).
#[must_use]
pub fn macos_sysctl_u32(name: &[u8]) -> Option<u32> {
  let mut val: u32 = 0;
  let mut len = size_of::<u32>();
  let ret = unsafe {
    sysctlbyname(
      name.as_ptr().cast::<c_char>(),
      ptr::from_mut(&mut val).cast::<c_void>(),
      &mut len,
      ptr::null(),
      0,
    )
  };
  if ret == 0 && len == size_of::<u32>() {
    Some(val)
  } else {
    None
  }
}

/// Read a 64-bit integer sysctl by name (e.g. `b"hw.memsize\0"`).
fn macos_sysctl_u64(name: &[u8]) -> Option<u64> {
  let mut val: u64 = 0;
  let mut len = size_of::<u64>();
  let ret = unsafe {
    sysctlbyname(
      name.as_ptr().cast::<c_char>(),
      ptr::from_mut(&mut val).cast::<c_void>(),
      &mut len,
      ptr::null(),
      0,
    )
  };
  if ret == 0 && len == size_of::<u64>() {
    Some(val)
  } else {
    None
  }
}

const HOST_VM_INFO64: c_int = 4;

/// Mirror of the kernel's `vm_statistics64` (`<mach/vm_statistics.h>`).
/// `natural_t` is `u32`. Declared in full so the integer count handed to
/// `host_statistics64` matches the kernel's expectation exactly (38 ints).
#[repr(C)]
#[derive(Default)]
#[allow(dead_code, reason = "most fields exist only to fix the struct layout")]
struct VmStatistics64 {
  free_count:                             u32,
  active_count:                           u32,
  inactive_count:                         u32,
  wire_count:                             u32,
  zero_fill_count:                        u64,
  reactivations:                          u64,
  pageins:                                u64,
  pageouts:                               u64,
  faults:                                 u64,
  cow_faults:                             u64,
  lookups:                                u64,
  hits:                                   u64,
  purges:                                 u64,
  purgeable_count:                        u32,
  speculative_count:                      u32,
  decompressions:                         u64,
  compressions:                           u64,
  swapins:                                u64,
  swapouts:                               u64,
  compressor_page_count:                  u32,
  throttled_count:                        u32,
  external_page_count:                    u32,
  internal_page_count:                    u32,
  total_uncompressed_pages_in_compressor: u64,
}

/// Returns `(used_bytes, total_bytes)` of physical memory.
///
/// `total` is `hw.memsize`. `used` approximates Activity Monitor's "Memory
/// Used" as `(active + wired + compressed) * page_size`; inactive and free
/// pages are treated as available. This is an approximation and the exact
/// formula can be tuned against real hardware.
#[must_use]
pub fn macos_meminfo() -> Option<(u64, u64)> {
  let total = macos_sysctl_u64(b"hw.memsize\0")?;
  // Apple Silicon uses 16 KiB pages; fall back to that if the sysctl is absent.
  let page = u64::from(macos_sysctl_u32(b"hw.pagesize\0").unwrap_or(16384));

  let mut vm = VmStatistics64::default();
  let mut count = (size_of::<VmStatistics64>() / size_of::<i32>()) as u32;
  let kr = unsafe {
    host_statistics64(
      mach_host_self(),
      HOST_VM_INFO64,
      ptr::from_mut(&mut vm).cast::<i32>(),
      &mut count,
    )
  };
  if kr != 0 {
    // Mach call failed; report total with unknown usage rather than erroring.
    return Some((0, total));
  }

  let used = (u64::from(vm.active_count)
    + u64::from(vm.wire_count)
    + u64::from(vm.compressor_page_count))
    * page;
  Some((used.min(total), total))
}

/// `struct timeval` (`time_t` is 64-bit, `suseconds_t` is 32-bit on macOS).
#[repr(C)]
#[derive(Default)]
#[allow(
  dead_code,
  reason = "tv_usec/_pad exist only to fix the struct layout"
)]
struct Timeval {
  tv_sec:  i64,
  tv_usec: i32,
  _pad:    i32,
}

const CTL_KERN: c_int = 1;
const KERN_BOOTTIME: c_int = 21;

/// Returns the system uptime in seconds (`now - kern.boottime`).
#[must_use]
pub fn macos_uptime_secs() -> Option<u64> {
  let mib = [CTL_KERN, KERN_BOOTTIME];
  let mut boot = Timeval::default();
  let mut len = size_of::<Timeval>();
  let ret = unsafe {
    sysctl(
      mib.as_ptr(),
      mib.len() as u32,
      ptr::from_mut(&mut boot).cast::<c_void>(),
      &mut len,
      ptr::null(),
      0,
    )
  };
  if ret != 0 {
    return None;
  }

  let mut now = Timeval::default();
  if unsafe { gettimeofday(&mut now, ptr::null_mut()) } != 0 {
    return None;
  }

  let secs = now.tv_sec - boot.tv_sec;
  if secs < 0 {
    None
  } else {
    // `secs` is non-negative here, so this is the exact value as `u64`.
    Some(secs.unsigned_abs())
  }
}
