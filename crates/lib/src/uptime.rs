#[cfg(target_os = "linux")] use core::mem::MaybeUninit;

#[cfg(target_os = "linux")] use crate::syscall::sys_sysinfo;
use crate::{Error, StackWriter};

/// Gets the current system uptime.
///
/// # Errors
///
/// Returns an error if the system uptime cannot be retrieved.
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn write_uptime(w: &mut StackWriter) -> Result<(), Error> {
  #[cfg(target_os = "linux")]
  let uptime_seconds = {
    let mut info = MaybeUninit::uninit();
    if unsafe { sys_sysinfo(info.as_mut_ptr()) } != 0 {
      return Err(Error::last_os_error());
    }
    #[allow(clippy::cast_sign_loss)]
    unsafe {
      info.assume_init().uptime as u64
    }
  };
  #[cfg(target_os = "macos")]
  let uptime_seconds =
    crate::syscall::macos_uptime_secs().ok_or(Error::OsError(0))?;

  let days = uptime_seconds / 86400;
  let hours = (uptime_seconds / 3600) % 24;
  let minutes = (uptime_seconds / 60) % 60;
  let mut any = false;

  if days > 0 {
    w.push_u64(days);
    w.push_str(if days == 1 { " day" } else { " days" });
    any = true;
  }
  if hours > 0 {
    if any {
      w.push_str(", ");
    }
    w.push_u64(hours);
    w.push_str(if hours == 1 { " hour" } else { " hours" });
    any = true;
  }
  if minutes > 0 {
    if any {
      w.push_str(", ");
    }
    w.push_u64(minutes);
    w.push_str(if minutes == 1 { " minute" } else { " minutes" });
    any = true;
  }
  if !any {
    w.push_str("less than a minute");
  }

  Ok(())
}
