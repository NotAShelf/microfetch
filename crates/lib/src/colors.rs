include!(concat!(env!("OUT_DIR"), "/colors_config.rs"));

use alloc::string::String;

/// Color codes for terminal output
pub struct Colors {
  pub reset:   &'static str,
  pub blue:    &'static str,
  pub cyan:    &'static str,
  pub green:   &'static str,
  pub yellow:  &'static str,
  pub red:     &'static str,
  pub magenta: &'static str,
  pub l1:      &'static str,
  pub l2:      &'static str,
  pub icon:    &'static str,
  pub key:     &'static str,
  pub value:   &'static str,
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
        l1:      "",
        l2:      "",
        icon:    "",
        key:     "",
        value:   "",
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
        l1:      COLOR_LOGO1,
        l2:      COLOR_LOGO2,
        icon:    COLOR_ICON,
        key:     COLOR_KEY,
        value:   COLOR_VAL,
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

#[must_use]
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn print_dots() -> String {
  const GLYPH: &str = "";

  let colors = if is_no_color() {
    Colors::new(true)
  } else {
    Colors::new(false)
  };

  // Pre-calculate capacity: 6 color codes + "  " (glyph + 2 spaces) per color
  let capacity = colors.blue.len()
    + colors.cyan.len()
    + colors.green.len()
    + colors.yellow.len()
    + colors.red.len()
    + colors.magenta.len()
    + colors.reset.len()
    + (GLYPH.len() + 2) * 6;

  let mut result = String::with_capacity(capacity);
  result.push_str(colors.blue);
  result.push_str(GLYPH);
  result.push_str("  ");
  result.push_str(colors.cyan);
  result.push_str(GLYPH);
  result.push_str("  ");
  result.push_str(colors.green);
  result.push_str(GLYPH);
  result.push_str("  ");
  result.push_str(colors.yellow);
  result.push_str(GLYPH);
  result.push_str("  ");
  result.push_str(colors.red);
  result.push_str(GLYPH);
  result.push_str("  ");
  result.push_str(colors.magenta);
  result.push_str(GLYPH);
  result.push_str("  ");
  result.push_str(colors.reset);

  result
}
