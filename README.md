## bandwhich

![demo](demo.gif)

This is a CLI utility for displaying current network utilization by process, connection and remote IP/hostname

### How does it work?
`bandwhich` sniffs a given network interface and records IP packet size, cross referencing it with the `/proc` filesystem on linux or `lsof` on macOS. It is responsive to the terminal window size, displaying less info if there is no room for it. It will also attempt to resolve ips to their host name in the background using reverse DNS on a best effort basis.

### Installation

#### Arch Linux

```
pacman -S bandwhich
```

#### Nix/NixOS

`bandwhich` is available in [`nixpkgs`](https://github.com/nixos/nixpkgs/blob/master/pkgs/tools/networking/bandwhich/default.nix), and can be installed, for example, with `nix-env`:

```
nix-env -iA nixpkgs.bandwhich
```

#### Void Linux

```
xbps-install -S bandwhich
```

#### Fedora

`bandwhich` is available in [COPR](https://copr.fedorainfracloud.org/coprs/atim/bandwhich/), and can be installed via DNF:

```
sudo dnf copr enable atim/bandwhich -y && sudo dnf install bandwhich
```

#### macOS

```
brew install bandwhich
```

#### Other Linux flavours

`bandwhich` can be installed using the Rust package manager, cargo. If it's not in your distro repositories or the available version is too old, you can install it via [rustup](https://rustup.rs/). You can find additional installation instructions [here](https://doc.rust-lang.org/book/ch01-01-installation.html).

The minimum supported Rust version is **1.39.0**.

```
cargo install bandwhich
```

This installs `bandwhich` to `~/.cargo/bin/bandwhich` but you need root priviliges to run `bandwhich`. To fix that, there are a few options:
- Give the executable elevated permissions: `sudo setcap cap_net_raw,cap_net_admin+ep ~/.cargo/bin/bandwhich` (not 100% the same as `sudo`, see explanation below)
- Run `sudo ~/.cargo/bin/bandwhich` instead of just `bandwhich`
- Create a symlink: `sudo ln -s ~/.cargo/bin/bandwhich /usr/local/bin/` (or another path on root's PATH)
- Set root's PATH to match your own `sudo env "PATH=$PATH" bandwhich`
- Pass the desired target directory to cargo: `sudo cargo install bandwhich --root /usr/local/bin/`

#### Download a prebuilt binary
If you're on linux, you could also get the generic binary from the releases.

#### Windows
Unfortunately, windows is not supported at the moment - if you'd like to contribute a windows port, it would be very much welcome.

### Usage
```
USAGE:
    bandwhich [FLAGS] [OPTIONS]

FLAGS:
    -a, --addresses      Show remote addresses table only
    -c, --connections    Show connections table only
    -h, --help           Prints help information
    -n, --no-resolve     Do not attempt to resolve IPs to their hostnames
    -p, --processes      Show processes table only
    -r, --raw            Machine friendlier output
    -V, --version        Prints version information

OPTIONS:
    -i, --interface <interface>    The network interface to listen on, eg. eth0
```

**Note that since `bandwhich` sniffs network packets, it requires root privileges** - so you might want to use it with (for example) `sudo`.

On Linux, you can give the `bandwhich` binary a permanent capability to use the required privileges, so that you don't need to use `sudo bandwhich` anymore:

```bash
sudo setcap cap_net_raw,cap_net_admin+ep "$HOME/.cargo/bin/bandwhich"
```

This is not 100% the same as running `bandwhich` as `sudo`. The above `setcap` commands gives `bandwhich` capability to sniff network packets. In order to run, `bandwhich` also needs the ability to read `procfs`. Normally processes can read `procfs`, however, if your system has [hidepid](https://linux-audit.com/linux-system-hardening-adding-hidepid-to-proc/) enabled, this assumption might not hold.

### raw_mode
`bandwhich` also supports an easier-to-parse mode that can be piped or redirected to a file. For example, try:
```
bandwhich --raw | grep firefox
```
### Contributing
Contributions of any kind are very welcome. If you'd like a new feature (or found a bug), please open an issue or a PR.

To set up your development environment:
1. Clone the project
2. `cargo run`, or if you prefer `cargo run -- -i <network interface name>` (you can often find out the name with `ifconfig` or `iwconfig`). You might need root privileges to run this application, so be sure to use (for example) sudo.

To run tests: `cargo test`

Note that at the moment the tests do not test the os layer (anything in the `os` folder).

If you are stuck, unsure about how to approach an issue or would like some guidance, you are welcome to contact: aram@poor.dev

### License
MIT
