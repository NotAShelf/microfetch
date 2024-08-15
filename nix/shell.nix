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
    gnuplot

    rust-analyzer-unwrapped
    rustfmt
    clippy
  ];

  env.RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
}
