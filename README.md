## what
...is taking up my bandwidth?!

![demo](demo.gif)

This is a CLI utility for displaying current network utilization by process, connection and remote IP/hostname

### How does it work?
`what` sniffs a given network interface and records IP packet size, cross referencing it with the `/proc` filesystem on linux or `lsof` on MacOS. It is responsive to the terminal window size, displaying less info if there is no room for it. It will also attempt to resolve ips to their host name in the background using reverse DNS on a best effort basis.

### Installation

#### Arch Linux

```
yay -S what
```

#### MacOS and other Linux flavours

```
cargo install what
```

Windows is not supported at the moment - if you'd like to contribute a windows port, it would be very much welcome.

### Usage
```
USAGE:
    what [FLAGS] --interface <interface>

FLAGS:
    -h, --help          Prints help information
    -n, --no-resolve    Do not attempt to resolve IPs to their hostnames
    -r, --raw           Machine friendlier output
    -V, --version       Prints version information

OPTIONS:
    -i, --interface <interface>    The network interface to listen on, eg. eth0
```

Note that since `what` sniffs network packets, it requires root privileges - so you might want to use it with (for example) `sudo`.

### raw_mode
`what` also supports an easier-to-parse mode that can be piped or redirected to a file. For example, try:
```
what -i eth0 --raw | grep firefox
```
### Contributing
Contributions of any kind are very welcome. If you'd like a new feature (or found a bug), please open an issue or a PR.

### License
MIT
