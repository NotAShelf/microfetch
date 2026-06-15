{
  lib,
  stdenv,
  mkShell,
  cargo,
  rustc,
  mold,
  clang,
  rust-analyzer,
  rustfmt,
  clippy,
  taplo,
  gnuplot,
}:
mkShell {
  name = "microfetch";
  strictDeps = true;
  nativeBuildInputs =
    [
      cargo
      rustc
      clang

      rust-analyzer
      (rustfmt.override {asNightly = true;})
      clippy
      taplo

      gnuplot # for Criterion.rs plots
    ]
    # mold is the Linux linker wrapper; macOS uses the default linker.
    ++ lib.optionals stdenv.isLinux [mold];
}
