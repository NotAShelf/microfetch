{
  mkShell,
  rust-analyzer-unwrapped,
  rustfmt,
  clippy,
  cargo,
  rustc,
  gcc,
  rustPlatform,
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
  ];

  env.RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
}
