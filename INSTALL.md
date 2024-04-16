# Installation

- [Installation](#installation)
  - [Arch Linux](#arch-linux)
  - [Exherbo Linux](#exherbo-linux)
  - [Nix/NixOS](#nixnixos)
  - [Void Linux](#void-linux)
  - [Fedora](#fedora)
  - [macOS/Linux (using Homebrew)](#macoslinux-using-homebrew)
  - [macOS (using MacPorts)](#macos-using-macports)
  - [FreeBSD](#freebsd)
  - [Cargo](#cargo)

## Arch Linux

```
pacman -S bandwhich
```

## Exherbo Linux

`bandwhich` is available in [rust repository](https://gitlab.exherbo.org/exherbo/rust/-/tree/master/packages/sys-apps/bandwhich), and can be installed via `cave`:

```
cave resolve -x repository/rust
cave resolve -x bandwhich
```

## Nix/NixOS

`bandwhich` is available in [`nixpkgs`](https://github.com/nixos/nixpkgs/blob/master/pkgs/tools/networking/bandwhich/default.nix), and can be installed, for example, with `nix-env`:

```
nix-env -iA nixpkgs.bandwhich
```

## Void Linux

```
xbps-install -S bandwhich
```

## Fedora

`bandwhich` is available in [COPR](https://copr.fedorainfracloud.org/coprs/atim/bandwhich/), and can be installed via DNF:

```
sudo dnf copr enable atim/bandwhich -y && sudo dnf install bandwhich
```

## macOS/Linux (using Homebrew)

```
brew install bandwhich
```

## macOS (using MacPorts)

```
sudo port selfupdate
sudo port install bandwhich
```

## FreeBSD

```
pkg install bandwhich
```

or

```
cd /usr/ports/net-mgmt/bandwhich && make install clean
```

## Cargo

Regardless of OS, you can always fallback to the Rust package manager, `cargo`.
For installation instructions of the Rust toolchain, see [here](https://www.rust-lang.org/tools/install).

```
cargo install bandwhich
```
