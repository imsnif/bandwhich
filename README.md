## what
...is taking up my bandwidth?!

![demo](demo.gif)

(display current network utilization by process, connection and remote IP/hostname)

This is my first attempt at Rust. :)

### How does it work?
`what` sniffs a given network interface and records IP packet size, cross referencing it with the `/proc` filesystem.

`what` is responsive to the terminal window size, displaying less info if there is no room for it.

`what` will attempt to resolve ips to their host name in the background using reverse DNS on a best effort basis.

### Installation
At the moment, `what` is available through Cargo as a binary package.

### Usage
`what -i <interface-name>` eg. `what -i eth0`

Note that since `what` sniffs network packets, it requires root privileges - so you might want to use it with (for example) `sudo`.
