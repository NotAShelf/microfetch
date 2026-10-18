#[cfg(target_os = "linux")] use core::mem::MaybeUninit;

use crate::StackWriter;
#[cfg(target_os = "linux")] use crate::syscall::sys_sysinfo;

/// Writes the current system uptime, or nothing if it cannot be retrieved.
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn write_uptime(w: &mut StackWriter) {
  #[cfg(target_os = "linux")]
  let uptime_seconds = {
    let mut info = MaybeUninit::uninit();
    if unsafe { sys_sysinfo(info.as_mut_ptr()) } != 0 {
      return;
    }
    let Ok(seconds) = u64::try_from(unsafe { info.assume_init() }.uptime)
    else {
      return;
    };
    seconds
  };
  #[cfg(target_os = "macos")]
  let Some(uptime_seconds) = crate::syscall::macos_uptime_secs() else {
    return;
  };

  let days = uptime_seconds / 86400;
  let hours = (uptime_seconds / 3600) % 24;
  let minutes = (uptime_seconds / 60) % 60;
  let mut any = days > 0;

  if any {
    w.push_u64(days);
    w.push_str(if days == 1 { " day" } else { " days" });
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
}
