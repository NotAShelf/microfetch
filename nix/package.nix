{
  lib,
  stdenv,
  rustPlatform,
  llvm,
  clang,
  wild,
  # Check deps
  versionCheckHook,
}: let
  pname = "microfetch";
  toml = (lib.importTOML ../Cargo.toml).workspace.package;
  inherit (toml) version;
  # On Linux the build drives the wild linker with LLVM/clang. macOS cannot
  # link statically and uses the default Apple clang stdenv with libSystem.
  stdenv' =
    if stdenv.isDarwin
    then rustPlatform.buildRustPackage
    else rustPlatform.buildRustPackage.override {inherit (llvm) stdenv;};

  hasWild =
    stdenv.hostPlatform.isLinux
    && (stdenv.hostPlatform.isx86_64 || stdenv.hostPlatform.isAarch64);
in
  stdenv' (finalAttrs: {
    __structuredAttrs = true;

    inherit pname version;
    src = let
      fs = lib.fileset;
      s = ../.;
    in
      fs.toSource {
        root = s;
        fileset = fs.unions [
          (s + /.cargo)
          (s + /crates)
          (s + /microfetch)
          (s + /Cargo.lock)
          (s + /Cargo.toml)
        ];
      };

    nativeBuildInputs = lib.optionals hasWild [wild clang];

    env = lib.optionalAttrs hasWild {
      RUSTFLAGS = "-Cforce-unwind-tables=no -Clinker=${clang}/bin/clang -Clink-arg=--ld-path=${wild}/bin/wild";
    };

    cargoLock.lockFile = "${finalAttrs.src}/Cargo.lock";
    strictDeps = true;
    enableParallelBuilding = true;
    buildNoDefaultFeatures = true;

    doInstallCheck = true;
    nativeInstallCheckInputs = [versionCheckHook];

    meta = {
      description = "Microscopic fetch script in Rust, for NixOS systems";
      homepage = "https://github.com/NotAShelf/microfetch";
      license = lib.licenses.gpl3Only;
      platforms = lib.platforms.linux ++ ["aarch64-darwin"];
      maintainers = [lib.maintainers.NotAShelf];
      mainProgram = "microfetch";
    };
  })
