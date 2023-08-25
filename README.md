## bandwhich

![demo](demo.gif)

This is a CLI utility for displaying current network utilization by process, connection and remote IP/hostname

### How does it work?
`bandwhich` sniffs a given network interface and records IP packet size, cross referencing it with the `/proc` filesystem on linux, `lsof` on macOS, or using WinApi on windows. It is responsive to the terminal window size, displaying less info if there is no room for it. It will also attempt to resolve ips to their host name in the background using reverse DNS on a best effort basis.

### Installation

#### Download a prebuilt binary
If you're on linux, you can download the generic binary from the releases.

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

#### macOS/Linux (using Homebrew)

On Linux, make sure the install directory is added to `$PATH`. See [documentation](https://docs.brew.sh/Homebrew-on-Linux#install).
You may also want to [make `sudo` preserve your `$PATH` environment variable](https://unix.stackexchange.com/q/83191/375550).

```
brew install bandwhich
```

#### macOS (using MacPorts)

```
sudo port selfupdate
sudo port install bandwhich
```

#### FreeBSD

```
pkg install bandwhich
```

or

```
cd /usr/ports/net-mgmt/bandwhich && make install clean
```

#### Windows / Other Linux flavours

`bandwhich` can be installed using the Rust package manager, cargo. It might be in your distro repositories if you're on linux, or you can install it via [rustup](https://rustup.rs/). You can find additional installation instructions [here](https://doc.rust-lang.org/book/ch01-01-installation.html).

The minimum supported Rust version is **1.65.0**.

```
cargo install bandwhich
```

##### On Linux, after installing with cargo:
Cargo installs `bandwhich` to `~/.cargo/bin/bandwhich` but you need root privileges to run `bandwhich`. To fix that, there are a few options:
- Give the executable elevated permissions: ``sudo setcap cap_sys_ptrace,cap_dac_read_search,cap_net_raw,cap_net_admin+ep $(which bandwhich)`` 
- Run `sudo ~/.cargo/bin/bandwhich` instead of just `bandwhich`
- Create a symlink: `sudo ln -s ~/.cargo/bin/bandwhich /usr/local/bin/` (or another path on root's PATH)
- Set root's PATH to match your own: `sudo env "PATH=$PATH" bandwhich`
- Tell sudo to use your user's environment variables: `sudo -E bandwhich`
- Pass the desired target directory to cargo: `sudo cargo install bandwhich --root /usr/local/bin/`

##### On Windows, after installing with cargo:
You might need to first install [npcap](https://nmap.org/npcap/) for capturing packets on windows.

#### OpenWRT

To install `bandwhich` on OpenWRT, you'll need to compile a binary that would fit its processor architecture. This might mean you would have to cross compile if, for example, you're working on an `x86_64` and the OpenWRT is installed on an `arm7`.
Here is an example of cross compiling in this situation:

- Check the processor architecture of your router by using `uname -m`
- Clone the bandwhich repository `git clone https://github.com/imsnif/bandwhich`
- Install `cross` using `cargo install cross`
- build the `bandwhich` package using `cross build --target armv7-unknown-linux-musleabihf`
- Copy the binary files from `target/armv7-unknown-linux-musleabihf/debug/bandwhich` to the router using `scp` by running `scp bandwhich root@192.168.1.1:~/` (here, 192.168.1.1 would be the IP address of your router).
- Finally enter the router using ssh and run the binary directly with `./bandwhich`

### Usage
```
USAGE:
    bandwhich [FLAGS] [OPTIONS]

FLAGS:
    -a, --addresses            Show remote addresses table only
    -c, --connections          Show connections table only
    -h, --help                 Prints help information
    -n, --no-resolve           Do not attempt to resolve IPs to their hostnames
    -p, --processes            Show processes table only
    -r, --raw                  Machine friendlier output
    -s, --show-dns             Show DNS queries
    -t, --total-utilization    Show total (cumulative) usages
    -V, --version              Prints version information

OPTIONS:
    -i, --interface <interface>    The network interface to listen on, eg. eth0
    -d, --dns-server <dns-server>    A dns server ip to use instead of the system default
```

**Note that since `bandwhich` sniffs network packets, it requires root privileges** - so you might want to use it with (for example) `sudo`.

On Linux, you can give the `bandwhich` binary a permanent capability to use the required privileges, so that you don't need to use `sudo bandwhich` anymore:

```bash
sudo setcap cap_sys_ptrace,cap_dac_read_search,cap_net_raw,cap_net_admin+ep $(command -v bandwhich)
```
`cap_sys_ptrace,cap_dac_read_search` gives `bandwhich` capability to list `/proc/<pid>/fd/` and resolve symlinks in that directory. It needs this capability to determine which opened port belongs to which process. `cap_net_raw,cap_net_admin` gives `bandwhich` capability to capture packets on your system.


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
