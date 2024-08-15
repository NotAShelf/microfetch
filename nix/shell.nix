{
  mkShell,
  rust-analyzer-unwrapped,
  rustfmt,
  clippy,
  cargo,
  rustc,
  gcc,
  rustPlatform,
  gnuplot,
}:
mkShell {
  strictDeps = true;

  nativeBuildInputs = [
    cargo
    rustc
    gcc

    rust-analyzer-unwrapped
    rustfmt
    clippy

    gnuplot # For Criterion.rs plots
  ];

  env.RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
}
