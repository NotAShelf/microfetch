//! Runs microfetch once with hotpath profiling. The `microfetch` binary is
//! `no_std` with its own `_start`, so it cannot host hotpath; this runs the
//! same `microfetch_lib::run` from a regular std binary instead.

use std::{ffi::CString, os::unix::ffi::OsStringExt, ptr};

fn main() {
  let env: Vec<CString> = std::env::vars_os()
    .map(|(key, value)| {
      let mut entry = key.into_vec();
      entry.push(b'=');
      entry.extend(value.into_vec());
      CString::new(entry).expect("environment entries contain no NUL bytes")
    })
    .collect();
  let mut envp: Vec<*const u8> =
    env.iter().map(|entry| entry.as_ptr().cast()).collect();
  envp.push(ptr::null());
  let argv = [c"microfetch".as_ptr().cast::<u8>(), ptr::null()];

  // SAFETY: `envp` and `argv` are null-terminated arrays of C strings that
  // outlive the run.
  unsafe {
    microfetch_lib::init_env(envp.as_ptr());
    microfetch_lib::run(1, argv.as_ptr()).expect("microfetch failed");
  }
}
