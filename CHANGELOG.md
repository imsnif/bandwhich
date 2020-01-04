# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)

## [Unreleased]

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
