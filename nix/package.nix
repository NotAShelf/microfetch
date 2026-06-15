{
  lib,
  stdenv,
  rustPlatform,
  llvm,
}: let
  pname = "microfetch";
  toml = (lib.importTOML ../Cargo.toml).workspace.package;
  inherit (toml) version;
  # On Linux the build drives the mold linker wrapper, which expects the
  # LLVM/clang stdenv. macOS cannot link statically and uses the default
  # (Apple clang) stdenv to link against libSystem instead.
  stdenv' =
    if stdenv.isDarwin
    then rustPlatform.buildRustPackage
    else rustPlatform.buildRustPackage.override {inherit (llvm) stdenv;};
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
          (s + /scripts/ld-wrapper)
          (s + /Cargo.lock)
          (s + /Cargo.toml)
        ];
      };

    cargoLock.lockFile = "${finalAttrs.src}/Cargo.lock";
    enableParallelBuilding = true;
    buildNoDefaultFeatures = true;
    doCheck = false;
    strictDeps = true;

    meta = {
      description = "Microscopic fetch script in Rust, for NixOS systems";
      homepage = "https://github.com/NotAShelf/microfetch";
      license = lib.licenses.gpl3Only;
      # aarch64-darwin only: x86_64-darwin would mis-route to the Linux x86_64
      platforms = lib.platforms.linux ++ ["aarch64-darwin"];
      maintainers = [lib.maintainers.NotAShelf];
      mainProgram = "microfetch";
    };
  })
