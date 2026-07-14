{
  lib,
  stdenv,
  mkShell,
  cargo,
  rustc,
  wild,
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
    ++ lib.optionals stdenv.isLinux [wild];
}
