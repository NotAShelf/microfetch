# Microfetch

A stupidly simple fetch tool, written in Rust. Runs in a fraction of a second,
displays most nonsense people on r/unixporn care about. Aims to replace
fastfetch on my system, but probably not on yours. Though you are more than
welcome to use it on your system: it's fast.

![Demo](.github/assets/demo.png)

## Features

- Fast
- Really fast
- Minimal dependencies
- Actually very fast
- Cool NixOS logo (other, inferior, distros are not supported)
- Reliable detection of following info:
  - Hostname/Username
  - Kernel
    - Name
    - Version
    - Architecture
  - Current shell (from $SHELL)
  - WM/Compositor and display backend
  - Memory Usage/Total Memory
  - Storage Usage/Total Storage (for `/` only)
  - Shell Colors

## Customizing

You can't.

## License

Microfetch is licensed under [GPL3](LICENSE). See the license file for details.
