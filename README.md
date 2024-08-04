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
- Actually really fast
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
- Did I mention fast?

## Benchmarks

Microfetch's performance is mostly hardware-dependant, however, the overall
trend seems to be < 2ms on any modern (2015 and after) CPU. Below are the
benchmarks with Hyperfine on my desktop system.

| Command                     | Mean [ms] | Min [ms] | Max [ms] | Relative |
| :-------------------------- | --------: | -------: | -------: | -------: |
| `target/release/microfetch` | 1.3 ± 0.1 |      1.2 |      3.7 |     1.00 |

On an average configuration, this is roughly 25 times faster than fastfetch and
around 80 times faster than neofetch. Results, as stated above, may vary.

## Customizing

You can't.

### Why?

Customization, of any kind, is expensive: I could try reading environment
variables, parse command-line arguments or read a configuration file but all of
those increment execution time and resource consumption by a lot.

### Really?

To be fair, you _can_ customize Microfetch by... Well, patching it. It's not the
best way per se, but it will be the only way that does not compromise on speed.

## Contributing

I will, mostly, reject feature additions. This is not to say you should avoid
them altogether, as you might have a really good idea worth discussing but as a
general rule of thumb consider talking to me before creating a feature PR.

Contributions that help improve performance in specific areas of Microfetch are
welcome. Though, prepare to be bombarded with questions.

## License

Microfetch is licensed under [GPL3](LICENSE). See the license file for details.
