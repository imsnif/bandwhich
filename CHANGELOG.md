# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)

## [Unreleased]

### Fixed
*  Fix the loss of large, merged packets (https://github.com/imsnif/bandwhich/pull/158) - [@TheLostLambda](https://github.com/TheLostLambda)

## [0.12.0] - 2020-03-01

### Added
* Add custom error handling (https://github.com/imsnif/bandwhich/pull/104) - [@captain-yossarian](https://github.com/captain-yossarian)

## [0.11.0] - 2020-01-25

### Added
* List unknown processes in processes table as well (https://github.com/imsnif/bandwhich/pull/132) - [@jcfvalente](https://github.com/jcfvalente)
* New layout (https://github.com/imsnif/bandwhich/pull/139) - [@imsnif](https://github.com/imsnif)

## [0.10.0] - 2020-01-18

### Added
* Support Ipv6 (https://github.com/imsnif/bandwhich/pull/70) - [@zhangxp1998](https://github.com/zhangxp1998)
* Select tables to render from the CLI (https://github.com/imsnif/bandwhich/pull/107) - [@chobeat](https://github.com/chobeat)

### Fixed
* VPN traffic sniffing on mac (https://github.com/imsnif/bandwhich/pull/129) - [@zhangxp1998](https://github.com/zhangxp1998)

## [0.9.0] - 2020-01-14

### Added

* Paused UI by pressing <SPACE> key. Does not affect raw mode. (https://github.com/imsnif/bandwhich/pull/106) - [@zhangxp1998](https://github.com/zhangxp1998)
* Mention setcap option in linux permission error. (https://github.com/imsnif/bandwhich/pull/108) - [@Ma27](https://github.com/Ma27)
* Display weighted average bandwidth for the past 5 seconds. (https://github.com/imsnif/bandwhich/pull/77) - [@zhangxp1998](https://github.com/zhangxp1998) + [@imsnif](https://github.com/imsnif)
* FreeBSD support. (https://github.com/imsnif/bandwhich/pull/110) - [@Erk-](https://github.com/Erk-)
* Pause help text. (https://github.com/imsnif/bandwhich/pull/111) - [@imsnif](https://github.com/imsnif)

### Fixed

* Upgrade trust-dns-resolver. (https://github.com/imsnif/bandwhich/pull/105) - [@bigtoast](https://github.com/bigtoast)
* Do not listen on inactive interfaces. (https://github.com/imsnif/bandwhich/pull/116) - [@zhangxp1998](https://github.com/zhangxp1998)

## [0.8.0] - 2020-01-09

### Added
- Brew formula and installation instructions for macOS (https://github.com/imsnif/bandwhich/pull/75) - [@imbsky](https://github.com/imbsky)
- UI change: add spacing between up and down rates for readability (https://github.com/imsnif/bandwhich/pull/58) - [@Calinou](https://github.com/Calinou)
- Support for wireguard interfaces (eg. for VPNs) (https://github.com/imsnif/bandwhich/pull/98) - [@Ma27](https://github.com/Ma27)
- Void linux installation instructions (https://github.com/imsnif/bandwhich/pull/102) - [@jcgruenhage](https://github.com/jcgruenhage)
- Arch installation with pacman (https://github.com/imsnif/bandwhich/pull/103) - [@kpcyrd](https://github.com/kpcyrd)

### Fixed

- Fix string conversion error on macOS (https://github.com/imsnif/bandwhich/pull/79) - [@zhangxp1998](https://github.com/zhangxp1998)
- Proper fix for macos no-screen-of-death (https://github.com/imsnif/bandwhich/pull/83) - [@zhangxp1998](https://github.com/zhangxp1998) + [@imsnif](https://github.com/imsnif)
- Fix UDP traffic not displayed issue #81 with (https://github.com/imsnif/bandwhich/pull/82) - [@zhangxp1998](https://github.com/zhangxp1998)
- Fix mac build (https://github.com/imsnif/bandwhich/pull/93) - [@imsnif](https://github.com/imsnif)
- Better procfs error handling (https://github.com/imsnif/bandwhich/pull/88) - [@zhangxp1998](https://github.com/zhangxp1998)


## [0.7.0] - 2020-01-05

### Added

- Running instructions (sudo) for a cargo installation (https://github.com/imsnif/bandwhich/pull/42) - [@LoyVanBeek](https://github.com/LoyVanBeek) + [@filalex77](https://github.com/filalex77)
- `setcap` instructions for linux instead of using sudo (https://github.com/imsnif/bandwhich/pull/57) - [@Calinou](https://github.com/Calinou)
- Installation instructions for Nix/NixOS (https://github.com/imsnif/bandwhich/pull/32) - [@filalex77](https://github.com/filalex77)
- MSRV and cargo installation instructions (https://github.com/imsnif/bandwhich/pull/66) - [@ebroto](https://github.com/ebroto)

### Fixed

- Repository URLs in Cargo.toml (https://github.com/imsnif/bandwhich/pull/43) - [@MatthieuBizien](https://github.com/MatthieuBizien)
- Skip interfaces with error (https://github.com/imsnif/bandwhich/pull/49) - [@Grishy](https://github.com/Grishy)
- MacOS no-screen-of-death workaround (https://github.com/imsnif/bandwhich/pull/56) - [@zhangxp1998](https://github.com/zhangxp1998)
- Reduce CPU utilization on linux (https://github.com/imsnif/bandwhich/pull/68) - [@ebroto](https://github.com/ebroto)
- Informative sudo error message (https://github.com/imsnif/bandwhich/pull/67) - [@Tobbeman](https://github.com/Tobbeman)
- Foreground text color for non-black terminals (https://github.com/imsnif/bandwhich/pull/65) - [@niiiil](https://github.com/niiiil)
- Do not truncate process names on MacOS (https://github.com/imsnif/bandwhich/pull/63) - [@zhangxp1998](https://github.com/zhangxp1998)
- Refactor tests into shared functionality (https://github.com/imsnif/bandwhich/pull/55) - [@chobeat](https://github.com/chobeat)
- Error on 0 interfaces found (https://github.com/imsnif/bandwhich/pull/69) - [@imsnif](https://github.com/imsnif)
