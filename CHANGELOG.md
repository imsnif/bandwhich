# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)

## [Unreleased]

### Fixed

* Update CONTRIBUTING information #438 - @YJDoc2 @cyqsimon
* Fix new clippy lint #457 - @cyqsimon
* Apply new clippy lints #468 - @cyqsimon

### Changed

* Bump msrv to 1.75.0 #439 - @YJDoc2
* Replace `derivative` with `derive_more` #439 - @YJDoc2
* Add build optimizations for release binary #434 - @pando85
* Minor cleanup and optimisations #435 - @cyqsimon
* Bump `pnet` & `packet-builder` #444 - @cyqsimon
* Switch from anyhow to eyre #450 - @cyqsimon
* Manually bump all dependencies #456 - @cyqsimon
* Bump MSRV to 1.82.0 - @cyqsimon

## [0.23.1] - 2024-10-09

### Fixed

* CI: Use Powershell Compress-Archive to create Windows binary zip #424 - @cyqsimon
* Exit gracefully when there is a broken pipe error #429 - @sigmaSd
* Fix breaking changes of sysinfo crate #431 - @cyqsimon
* Fix `clippy::needless_lifetimes` warnings on nightly #432 - @cyqsimon

## [0.23.0] - 2024-08-17

### Fixed

* Remove redundant imports #377 - @cyqsimon
* CI: use GitHub API to exempt dependabot from changelog requirement #378 - @cyqsimon
* Remove unnecessary logging synchronisation #381 - @cyqsimon
* Apply suggestions from new clippy lint clippy::assigning_clones #382 - @cyqsimon
* Fix IPv6 socket detect logic #383 - @cyqsimon
* Support build for `target_os` `android` #384 - @flxo
* Fix Windows FP discrepancy issue in test #400 - @cyqsimon

### Added

* CI: include generated assets in release archive #359 - @cyqsimon
* Add PID column to the process table #379 - @notjedi
* CI: add builds for target `aarch64-linux-android` #384 - @flxo
* CI: Keep GitHub Actions up to date with GitHub's Dependabot #403 - @cclauss
* CI: Enable more cross-compiled builds #401 - @cyqsimon
* CI: use sccache to speed up CI #408 - @cyqsimon

### Changed

* CI: strip release binaries for all targets #358 - @cyqsimon
* Bump MSRV to 1.74 (required by clap 4.5; see #373)
* CI: Configure dependabot grouping #395 - @cyqsimon
* CI refactor #399 - @cyqsimon
* CI: Temporarily disable UI tests #406 - @cyqsimon
* Update README #407 - @cyqsimon
* Update usage in README #409 - @cyqsimon

### Removed

* CI: Remove musl-tools install step #402 - @cyqsimon

## [0.22.2] - 2024-01-28

### Added

* Generate completion & manpage #357 - @cyqsimon

## [0.22.1] - 2024-01-28

### Fixed

* Hot fix a Windows compile issue #356 - @cyqsimon

## [0.22.0] - 2024-01-28

### Added

* Log unresolved processes in more detail + general refactor #318 - @cyqsimon
* Display bandwidth in different unit families #328 - @cyqsimon
* CI: ensure a changelog entry exists for each PR #331 - @cyqsimon
* Show interface names #340 - @ilyes-ced

### Changed

* Table formatting logic overhaul #305 - @cyqsimon
* Refactor OsInputOutput (combine interfaces & frames into single Vec) #310 - @cyqsimon

### Removed

* Reorganise & cleanup packaging code/resources #329 - @cyqsimon

### Fixed

* Make logging race-free using a global lock & macro #309 - @cyqsimon
* Use once_cell::sync::Lazy to make regex usage more ergonomic #313 - @cyqsimon
* Fix vague CLI option documentation; closes #314 #316 - @cyqsimon

## [0.21.1] - 2023-10-16

### Fixed
* Ignore connections that fail parsing instead of panicking on BSD (https://github.com/imsnif/bandwhich/pull/288) - [@cyqsimon](https://github.com/cyqsimon)
* Add missing version flag to CLI (https://github.com/imsnif/bandwhich/pull/290) - [@tranzystorek-io](https://github.com/tranzystorek-io)
* Various minor codestyle changes - [@cyqsimon](https://github.com/cyqsimon)
* Handle IPv4-mapped IPv6 addresses when resolving connection owner (https://github.com/imsnif/bandwhich/commit/76956cf) - [@cyqsimon](https://github.com/cyqsimon)
* Bump `rustix` dependencies to fix a memory leak (https://github.com/imsnif/bandwhich/commit/bc10c07) - [@cyqsimon](https://github.com/cyqsimon)

### Added
* Logging infrastrure (https://github.com/imsnif/bandwhich/pull/302) - [@cyqsimon](https://github.com/cyqsimon)

## [0.21.0] - 2023-09-19

### Fixed
* Fixed resolv.conf errors on systems with trust-ad (https://github.com/imsnif/bandwhich/pull/201) - [@JoshLambda](https://github.com/JoshLambda)
* Fixed build issues by updating various dependencies
* migrate out-of-date dependency `structopt` to `clap` (https://github.com/imsnif/bandwhich/pull/285) - [@Liyixin95](https://github.com/Liyixin95)

## [0.20.0] - 2020-10-15

### Added
* New command line argument to explicitly specify a DNS server to use (https://github.com/imsnif/bandwhich/pull/193) - [@imsnif](https://github.com/imsnif)

## [0.19.0] - 2020-09-29

### Fixed
* Fixed resolv.conf parsing for rDNS in some cases (https://github.com/imsnif/bandwhich/pull/184) - [@Ma27](https://github.com/Ma27)
* Cross platform window resizing (fixes momentary UI break when resizing window on Windows) (https://github.com/imsnif/bandwhich/pull/186) - [@remgodow](https://github.com/remgodow)
* CI: build binaries using github actions (https://github.com/imsnif/bandwhich/pull/181) - [@remgodow](https://github.com/remgodow)
* Fix build on FreeBSD (https://github.com/imsnif/bandwhich/pull/189) - [@imsnif](https://github.com/imsnif)
* Upgrade TUI to latest version (https://github.com/imsnif/bandwhich/pull/190) - [@imsnif](https://github.com/imsnif)
* Try to reconnect to disconnected interfaces (https://github.com/imsnif/bandwhich/pull/191) - [@thepacketgeek](https://github.com/thepacketgeek)

## [0.18.1] - 2020-09-11

* HOTFIX: do not build windows build-dependencies on other platforms

## [0.18.0] - 2020-09-11

### Added
* Future windows infrastructure support (should not have any user facing effect) (https://github.com/imsnif/bandwhich/pull/179) - [@remgodow](https://github.com/remgodow)
* Windows build and run support (https://github.com/imsnif/bandwhich/pull/180) - [@remgodow](https://github.com/remgodow)

### Fixed
* Update and improve MAN page (https://github.com/imsnif/bandwhich/pull/182) - [@Nudin](https://github.com/Nudin)

## [0.17.0] - 2020-09-02

### Added
* Add delimiters between refreshes in raw mode for easier parsing (https://github.com/imsnif/bandwhich/pull/175) - [@sigmaSd](https://github.com/sigmaSd)

### Fixed
* Truncate Chinese characters properly (https://github.com/imsnif/bandwhich/pull/177) - [@zxlzy](https://github.com/zxlzy)
* Moved to mebi/gibi/tibi bytes to improve bandwidth accuracy and reduce ambiguity (https://github.com/imsnif/bandwhich/pull/178) - [@imsnif](https://github.com/imsnif)

## [0.16.0] - 2020-07-13

### Fixed
* Allow filtering by processes/connections/remote-ips in raw-mode (https://github.com/imsnif/bandwhich/pull/174) - [@sigmaSd](https://github.com/sigmaSd)
* Changed repository trunk branch to "main" instead of "master".

## [0.15.0] - 2020-05-23

### Added
* Ability to change the window layout with <TAB> (https://github.com/imsnif/bandwhich/pull/118) - [@Louis-Lesage](https://github.com/Louis-Lesage)
* Show duration of current capture when running in "total utilization" mode. - [@Eosis](https://github.com/Eosis)

### Fixed
* Add terabytes as a display unit (for cumulative mode) (https://github.com/imsnif/bandwhich/pull/168) - [@TheLostLambda](https://github.com/TheLostLambda)

## [0.14.0] - 2020-05-03

### Fixed
* HOTFIX: remove pnet_bandwhich_fork dependency and upgrade to working version of pnet + packet_builder instead (this should hopefully not change anything)

## [0.13.0] - 2020-04-05

### Added
* Hide DNS queries by default. This can be overridden with `-s, --show-dns` (https://github.com/imsnif/bandwhich/pull/161) - [@olesh0](https://github.com/olehs0)
* Show cumulative utilization in "total utilization" mode. Trigger with `-t, --total-utilization` (https://github.com/imsnif/bandwhich/pull/155) - [@TheLostLambda](https://github.com/TheLostLambda)

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
