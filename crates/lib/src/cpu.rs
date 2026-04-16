#[cfg(target_os = "linux")]
use crate::syscall::read_file_fast;
use crate::{Error, StackWriter};

/// Writes CPU model name (trimmed) to the writer.
#[cfg(target_os = "linux")]
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn write_cpu_name(w: &mut StackWriter) {
  write_model_name(w);
}

/// Writes CPU model name from `machdep.cpu.brand_string` (macOS),
/// e.g. `Apple M2 Pro`. Returns an empty string if unavailable.
#[cfg(target_os = "macos")]
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn write_cpu_name(w: &mut StackWriter) {
  let mut buf = [0u8; 128];
  if let Some(n) =
    crate::syscall::macos_sysctl_str(b"machdep.cpu.brand_string\0", &mut buf)
  {
    if let Ok(name) = core::str::from_utf8(&buf[..n]) {
      w.push_str(name);
    }
  }
}

/// Writes CPU core/thread info string.
///
/// Format: `{cores} cores ({p}p/{e}e), {threads} threads` on hybrid Intel,
/// `{cores} cores, {threads} threads` otherwise.
///
/// # Errors
///
/// Returns an error if the thread count cannot be determined.
#[cfg(target_os = "linux")]
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn write_cpu_cores(w: &mut StackWriter) -> Result<(), Error> {
  let threads = get_thread_count()?;
  let cores = get_core_count(threads);
  write_cores(w, cores, get_pe_cores(), threads);
  Ok(())
}

/// Writes CPU core/thread info via `sysctl` (macOS).
///
/// On Apple Silicon `hw.perflevel0`/`hw.perflevel1` expose the performance
/// (P) and efficiency (E) core counts respectively.
///
/// # Errors
///
/// Returns an error if the logical CPU count cannot be determined.
#[cfg(target_os = "macos")]
#[cfg_attr(feature = "hotpath", hotpath::measure)]
pub fn write_cpu_cores(w: &mut StackWriter) -> Result<(), Error> {
  use crate::syscall::macos_sysctl_u32;

  let threads =
    macos_sysctl_u32(b"hw.logicalcpu\0").ok_or(Error::OsError(0))?;
  let cores = macos_sysctl_u32(b"hw.physicalcpu\0").unwrap_or(threads);

  // Performance/efficiency split (Apple Silicon). Only reported when both
  // perf levels are present.
  let pe = match (
    macos_sysctl_u32(b"hw.perflevel0.physicalcpu\0"),
    macos_sysctl_u32(b"hw.perflevel1.physicalcpu\0"),
  ) {
    (Some(p), Some(e)) => Some((p, e)),
    _ => None,
  };

  write_cores(w, cores, pe, threads);
  Ok(())
}

/// Writes core/thread counts identically across platforms:
/// `{cores} cores ({p}p/{e}e), {threads} threads`, omitting the P/E group and
/// the thread suffix when not applicable.
fn write_cores(
  w: &mut StackWriter,
  cores: u32,
  pe: Option<(u32, u32)>,
  threads: u32,
) {
  w.push_u64(u64::from(cores));
  w.push_str(" cores");

  if let Some((p, e)) = pe {
    w.push_str(" (");
    w.push_u64(u64::from(p));
    w.push_str("p/");
    w.push_u64(u64::from(e));
    w.push_str("e)");
  }

  if threads != cores {
    w.push_str(", ");
    w.push_u64(u64::from(threads));
    w.push_str(" threads");
  }
}

/// Count online threads via `sched_getaffinity(2)`.
#[cfg(target_os = "linux")]
fn get_thread_count() -> Result<u32, Error> {
  let mut mask = [0u8; 128];
  let ret = unsafe {
    crate::syscall::sys_sched_getaffinity(0, mask.len(), mask.as_mut_ptr())
  };
  if ret < 0 {
    return Err(Error::from_raw_os_error(-ret));
  }

  #[allow(clippy::cast_sign_loss)]
  let bytes = ret as usize;
  let mut count = 0u32;
  for &byte in &mask[..bytes] {
    count += byte.count_ones();
  }
  Ok(count)
}

/// Derive physical core count from thread count and topology.
#[cfg(target_os = "linux")]
fn get_core_count(threads: u32) -> u32 {
  let Some(smt_width) =
    count_cpulist("/sys/devices/system/cpu/cpu0/topology/thread_siblings_list")
  else {
    return threads;
  };
  if smt_width == 0 {
    return threads;
  }
  threads / smt_width
}

/// Detect P-core and E-core counts via sysfs PMU device files, which is done
/// by reading `/sys/devices/cpu_core/cpus` and `/sys/devices/cpu_atom/cpus`.
#[cfg(target_os = "linux")]
fn get_pe_cores() -> Option<(u32, u32)> {
  let p = count_cpulist("/sys/devices/cpu_core/cpus")?;
  let e = count_cpulist("/sys/devices/cpu_atom/cpus").unwrap_or(0);
  if p > 0 || e > 0 { Some((p, e)) } else { None }
}

/// Parse a cpulist file and count listed CPUs.
#[cfg(target_os = "linux")]
fn count_cpulist(path: &str) -> Option<u32> {
  let mut buf = [0u8; 64];
  let n = read_file_fast(path, &mut buf).ok()?;
  let data = &buf[..n];

  let mut count = 0u32;
  let mut i = 0;
  while i < data.len() {
    // Parse start number
    let start = parse_num(data, &mut i);
    if i < data.len() && data[i] == b'-' {
      i += 1;
      let end = parse_num(data, &mut i);
      // The Kernel always emits ascending ranges, so end is always >= start
      // https://github.com/torvalds/linux/blob/v6.19/lib/vsprintf.c#L1276-L1303
      count += end - start + 1;
    } else {
      count += 1;
    }
    // Skip comma or newline
    if i < data.len() && (data[i] == b',' || data[i] == b'\n') {
      i += 1;
    }
  }
  Some(count)
}

/// Parse a decimal number from a byte slice, advancing the index.
#[cfg(target_os = "linux")]
fn parse_num(data: &[u8], i: &mut usize) -> u32 {
  let mut n = 0u32;
  while *i < data.len() && data[*i].is_ascii_digit() {
    n = n * 10 + u32::from(data[*i] - b'0');
    *i += 1;
  }
  n
}

/// Build `/sys/devices/system/cpu/cpu{n}/cpufreq/cpuinfo_max_freq` into buf,
/// returning the byte length written.
#[cfg(target_os = "linux")]
fn format_cpufreq_path(buf: &mut [u8; 64], cpu: u32) -> usize {
  const PREFIX: &[u8] = b"/sys/devices/system/cpu/cpu";
  const SUFFIX: &[u8] = b"/cpufreq/cpuinfo_max_freq";
  buf[..PREFIX.len()].copy_from_slice(PREFIX);
  let mut i = PREFIX.len();
  let mut tmp = [0u8; 3];
  let mut n = cpu;
  let mut digits = 0;
  loop {
    tmp[digits] = b'0' + (n % 10) as u8;
    digits += 1;
    n /= 10;
    if n == 0 {
      break;
    }
  }
  while digits > 0 {
    digits -= 1;
    buf[i] = tmp[digits];
    i += 1;
  }
  buf[i..i + SUFFIX.len()].copy_from_slice(SUFFIX);
  i + SUFFIX.len()
}

/// Read CPU frequency in MHz. Tries sysfs first, then cpuinfo data.
#[cfg(target_os = "linux")]
fn get_cpu_freq_mhz(cpuinfo: &[u8]) -> Option<u32> {
  // Read cpuinfo_max_freq across all CPUs (in kHz) and take the max so
  // heterogeneous (big.LITTLE) topologies report the performance cluster.
  let mut max_khz = 0u32;
  let mut path = [0u8; 64];
  for cpu in 0u32..64 {
    let n = format_cpufreq_path(&mut path, cpu);
    let p = match core::str::from_utf8(&path[..n]) {
      Ok(s) => s,
      Err(_) => continue,
    };
    let mut buf = [0u8; 32];
    let Ok(m) = read_file_fast(p, &mut buf) else {
      if cpu == 0 {
        continue;
      }
      break;
    };
    let mut khz = 0u32;
    for &b in &buf[..m] {
      if b.is_ascii_digit() {
        khz = khz * 10 + u32::from(b - b'0');
      }
    }
    if khz > max_khz {
      max_khz = khz;
    }
  }
  if max_khz > 0 {
    return Some(max_khz / 1000);
  }
  // Fall back to cpuinfo fields
  for key in &[
    b"cpu MHz" as &[u8],
    b"cpu MHz dynamic",
    b"cpu MHz static",
    b"CPU MHz",
    b"clock",
    // BogoMIPS on MIPS is calibrated to the clock frequency (unlike x86).
    b"BogoMIPS",
  ] {
    if let Some(val) = extract_field(cpuinfo, key) {
      // Parse integer part of the MHz value (e.g. "5200.00" -> 5200)
      let mut mhz = 0u32;
      for &b in val.as_bytes() {
        if b == b'.' {
          break;
        }
        if b.is_ascii_digit() {
          mhz = mhz * 10 + u32::from(b - b'0');
        }
      }

      // Octeon presets loops_per_jiffy to clock_rate/HZ, so its BogoMIPS is
      // exactly 2x the core clock, unlike the 1:1 of other MIPS.
      // https://github.com/torvalds/linux/blob/v6.19/arch/mips/cavium-octeon/csrc-octeon.c#L40
      if *key == b"BogoMIPS" && cpuinfo.windows(6).any(|w| w == b"Octeon") {
        mhz /= 2;
      }

      if mhz > 0 {
        return Some(mhz);
      }
    }
  }
  // SPARC exposes its clock as `Cpu0ClkTck : <hex>`,
  // which signifies ticks per second in hex.
  if let Some(val) = extract_field(cpuinfo, b"Cpu0ClkTck") {
    let mut hz = 0u64;
    let mut seen = false;
    for &b in val.as_bytes() {
      let d = match b {
        b'0'..=b'9' => Some(u64::from(b - b'0')),
        b'a'..=b'f' => Some(u64::from(b - b'a' + 10)),
        b'A'..=b'F' => Some(u64::from(b - b'A' + 10)),
        _ => None,
      };
      match d {
        Some(d) => {
          hz = hz * 16 + d;
          seen = true;
        },
        None if seen => break,
        None => {},
      }
    }
    if hz > 0 {
      #[allow(clippy::cast_possible_truncation)]
      return Some((hz / 1_000_000) as u32);
    }
  }
  None
}

/// Parse CPU model name from `/proc/cpuinfo` and write it directly.
/// Appends CPU frequency if available.
#[cfg(target_os = "linux")]
fn write_model_name(w: &mut StackWriter) {
  let mut buf = [0u8; 2048];
  let Ok(n) = read_file_fast("/proc/cpuinfo", &mut buf) else {
    return;
  };
  let data = &buf[..n];

  let name = extract_name(data);
  if name.is_none() && !write_dt_compatible(w) {
    return;
  }

  let mhz = get_cpu_freq_mhz(data);
  if let Some(name) = name {
    // x86 `model name` already ends in `@ <clock>GHz`, which hides the
    // ` CPU` that trim() would otherwise strip, so re-trim after cutting.
    let name = match (mhz, name.find(" @ ")) {
      (Some(_), Some(at)) => trim(&name[..at]),
      _ => name,
    };
    w.push_str(name);
  }

  if let Some(mhz) = mhz {
    w.push_str(" @ ");
    // Round to nearest 0.01 GHz, then split so carries (e.g. 1999 MHz)
    // roll into the integer part instead of overflowing the fraction.
    let rounded_centesimal = (mhz + 5) / 10;
    let ghz_int = rounded_centesimal / 100;
    let ghz_frac = rounded_centesimal % 100;
    w.push_u64(u64::from(ghz_int));
    w.push_byte(b'.');
    if ghz_frac < 10 {
      w.push_byte(b'0');
    }
    w.push_u64(u64::from(ghz_frac));
    w.push_str(" GHz");
  }
}

/// Extract a human-readable CPU name from cpuinfo fields.
#[cfg(target_os = "linux")]
fn extract_name(data: &[u8]) -> Option<&str> {
  for key in &[
    b"model name" as &[u8],
    b"Model Name",
    b"uarch",
    b"cpu model",
    b"isa",
    b"cpu",
    b"machine",
    b"vendor_id",
  ] {
    if let Some(val) = extract_field(data, key) {
      let trimmed = trim(val);
      if !trimmed.is_empty() {
        return Some(trimmed);
      }
    }
  }
  None
}

/// Write the `SoC` name from `/sys/firmware/devicetree/base/compatible`.
/// The file holds NUL-separated `vendor,model` strings from most-specific
/// (board) to most-generic (`SoC`); we take the last entry and write just
/// the model portion after the comma.
#[cfg(target_os = "linux")]
fn write_dt_compatible(w: &mut StackWriter) -> bool {
  let mut buf = [0u8; 256];
  let Ok(n) =
    read_file_fast("/sys/firmware/devicetree/base/compatible", &mut buf)
  else {
    return false;
  };
  // Drop the terminating NUL so the rposition below locates the entry
  // separator rather than the end-of-string marker.
  let end = if n > 0 && buf[n - 1] == 0 { n - 1 } else { n };
  let data = &buf[..end];
  let start = data.iter().rposition(|&b| b == 0).map_or(0, |p| p + 1);
  let entry = &data[start..];
  let Some(comma) = entry.iter().position(|&b| b == b',') else {
    return false;
  };
  let Ok(model) = core::str::from_utf8(&entry[comma + 1..]) else {
    return false;
  };
  if model.is_empty() {
    return false;
  }
  w.push_str(model);
  true
}

/// Extract value of first occurrence of `key` in cpuinfo.
#[cfg(target_os = "linux")]
fn extract_field<'a>(data: &'a [u8], key: &[u8]) -> Option<&'a str> {
  let mut i = 0;
  while i < data.len() {
    let remaining = &data[i..];
    let eol = remaining
      .iter()
      .position(|&b| b == b'\n')
      .unwrap_or(remaining.len());
    let line = &remaining[..eol];

    if line.starts_with(key) {
      let mut p = key.len();
      while p < line.len() && (line[p] == b'\t' || line[p] == b' ') {
        p += 1;
      }
      if p < line.len() && line[p] == b':' {
        p += 1;
        while p < line.len() && line[p] == b' ' {
          p += 1;
        }
        // SAFETY: cpuinfo fields are ASCII
        return Some(unsafe { core::str::from_utf8_unchecked(&line[p..]) });
      }
    }

    i += eol + 1;
  }
  None
}

/// Strip noise from model names.
#[cfg(target_os = "linux")]
fn trim(name: &str) -> &str {
  let b = name.as_bytes();
  let mut end = b.len();

  while end > 0 && b[end - 1].is_ascii_whitespace() {
    end -= 1;
  }

  if end >= 10 && &b[end - 10..end] == b" Processor" {
    end -= 10;
  } else if end >= 4 && &b[end - 4..end] == b" CPU" {
    end -= 4;
  }
  while end > 0 && b[end - 1].is_ascii_whitespace() {
    end -= 1;
  }

  if end >= 3 && &b[end - 3..end] == b"(R)" {
    end -= 3;
  } else if end >= 4
    && (&b[end - 4..end] == b"(TM)" || &b[end - 4..end] == b"(tm)")
  {
    end -= 4;
  }
  while end > 0 && b[end - 1].is_ascii_whitespace() {
    end -= 1;
  }

  if end > 7 && &b[end - 5..end] == b"-Core" {
    let mut p = end - 5;
    while p > 0 && b[p - 1].is_ascii_digit() {
      p -= 1;
    }
    if p > 0 && b[p - 1] == b' ' {
      end = p - 1;
    }
  }
  while end > 0 && b[end - 1].is_ascii_whitespace() {
    end -= 1;
  }

  let mut start = 0;
  while start < end && b[start].is_ascii_whitespace() {
    start += 1;
  }

  &name[start..end]
}
