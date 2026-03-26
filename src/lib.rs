pub mod colors;
pub mod desktop;
pub mod release;
pub mod syscall;
pub mod system;
pub mod uptime;

use std::{ffi::CStr, mem::MaybeUninit};

use crate::syscall::{UtsNameBuf, sys_uname};

/// Wrapper for `utsname` with safe accessor methods
pub struct UtsName(UtsNameBuf);

impl UtsName {
  /// Calls `uname(2)` syscall and returns a `UtsName` wrapper
  ///
  /// # Errors
  ///
  /// Returns an error if the `uname` syscall fails
  pub fn uname() -> Result<Self, std::io::Error> {
    let mut uts = MaybeUninit::uninit();
    if unsafe { sys_uname(uts.as_mut_ptr()) } != 0 {
      return Err(std::io::Error::last_os_error());
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
