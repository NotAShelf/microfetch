use crate::StackWriter;

/// Color codes for terminal output
pub struct Colors {
  pub reset:   &'static str,
  pub blue:    &'static str,
  pub cyan:    &'static str,
  pub green:   &'static str,
  pub yellow:  &'static str,
  pub red:     &'static str,
  pub magenta: &'static str,
}

impl Colors {
  #[must_use]
  pub const fn new(is_no_color: bool) -> Self {
    if is_no_color {
      Self {
        reset:   "",
        blue:    "",
        cyan:    "",
        green:   "",
        yellow:  "",
        red:     "",
        magenta: "",
      }
    } else {
      Self {
        reset:   "\x1b[0m",
        blue:    "\x1b[34m",
        cyan:    "\x1b[36m",
        green:   "\x1b[32m",
        yellow:  "\x1b[33m",
        red:     "\x1b[31m",
        magenta: "\x1b[35m",
      }
    }
  }
}

use core::sync::atomic::{AtomicBool, Ordering};

// Check if NO_COLOR is set (only once, lazily)
// Only presence matters; value is irrelevant per the NO_COLOR spec
static NO_COLOR_CHECKED: AtomicBool = AtomicBool::new(false);
static NO_COLOR_SET: AtomicBool = AtomicBool::new(false);

/// Checks if `NO_COLOR` environment variable is set.
pub(crate) fn is_no_color() -> bool {
  // Fast path: already checked
  if NO_COLOR_CHECKED.load(Ordering::Acquire) {
    return NO_COLOR_SET.load(Ordering::Relaxed);
  }

  // Slow path: check environment
  let is_set = crate::env_exists("NO_COLOR");
  NO_COLOR_SET.store(is_set, Ordering::Relaxed);
  NO_COLOR_CHECKED.store(true, Ordering::Release);
  is_set
}

#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn write_dots(w: &mut StackWriter, colors: &Colors) {
  for c in [
    colors.blue,
    colors.cyan,
    colors.green,
    colors.yellow,
    colors.red,
    colors.magenta,
  ] {
    w.push_str(c);
    w.push_str("●  ");
  }
  w.push_str(colors.reset);
}
