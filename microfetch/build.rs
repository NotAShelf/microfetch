use std::{env, io, process::Command};

fn main() -> io::Result<()> {
  let rustc = env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
  let version = Command::new(rustc).arg("-vV").output()?;
  let nightly = String::from_utf8_lossy(&version.stdout)
    .lines()
    .any(|line| {
      line.starts_with("release:")
        && (line.contains("-nightly") || line.contains("-dev"))
    });

  println!("cargo::rustc-check-cfg=cfg(nightly)");
  if nightly {
    println!("cargo::rustc-cfg=nightly");
  }

  // These flags only apply to the microfetch binary, not to proc-macro crates
  // or other host-compiled artifacts.

  // macOS cannot link statically, requires the standard C runtime startup, and
  // uses Mach-O (not ELF), so none of the flags below apply. The default
  // linker driver produces a correct (and ad-hoc code-signed) binary there.
  let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
  if target_os == "macos" {
    return Ok(());
  }

  // No C runtime, we provide _start ourselves
  println!("cargo:rustc-link-arg-bin=microfetch=-nostartfiles");
  // Fully static, no dynamic linker, no .interp/.dynsym/.dynamic overhead
  println!("cargo:rustc-link-arg-bin=microfetch=-static");
  // Clang ignores Rust's PIE selection flag for fully static links.
  println!(
    "cargo:rustc-link-arg-bin=microfetch=-Wno-unused-command-line-argument"
  );
  // Remove unreferenced input sections
  println!("cargo:rustc-link-arg-bin=microfetch=-Wl,--gc-sections");
  // Strip all symbol table entries
  if std::env::var("DEBUG").as_deref() != Ok("true") {
    println!("cargo:rustc-link-arg-bin=microfetch=-Wl,--strip-all");
  }
  // Omit the .note.gnu.build-id section
  println!("cargo:rustc-link-arg-bin=microfetch=-Wl,--build-id=none");
  // Disable RELRO (removes relro_padding)
  println!("cargo:rustc-link-arg-bin=microfetch=-Wl,-z,norelro");

  Ok(())
}
