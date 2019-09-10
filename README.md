### what
Display current network utilization by process and connection.

This is my first attempt at Rust and is still very much a WIP. :)

### How does it work?
`what` sniffs a given network interface (provided as the second cli argument, eg. `what eth0`) and records packet size, cross referencing it with the `/proc` filesystem.

Currently, it relies on the display loop to reset its state every second and thus always display the bandwidth per second.

At the moment there is only a linux implementation but the tests should (hopefully!) run on all platforms.
