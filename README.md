# kvos

This is an operating system we implemented in Lehigh University's operating system implementation class.
The contributors are listed [here](./CONTRIBUTING.md)

## Getting started

- `setup.sh` will install all necessary dependencies on Ubuntu if you have yet to build with nightly on your machine
- run `cargo build` to build and `cargo test` to test

## Dependencies

- [qemu](https://www.qemu.org/download/)
- [rust](https://www.rust-lang.org/tools/install)
- [nasm](https://github.com/netwide-assembler/nasm#nasm-the-netwide-assembler)
- [grub](https://wiki.osdev.org/GRUB_2)

## Setup

Run:

```
rustup override set nightly
rustup component add rust-src
rustup target add x86_64-unknown-none
create_disks.sh
```

Then `cargo build`.

## Running

`cargo run` will run qemu. `cargo run -- -h` gives command line arguments.
Use `cargo run -- [options]` to run with any options. One option of note is
`-g` which will use gtk instead of sdl for displaying.

## Debugging

Run `cargo run -- -d`.
The iso image and kernel binary will be printed out.
Run `gdb [kernel binary name] -ex "target remote:1234"`.
Step through the program.
If you need to debug a test, set the timeout in `bootloader/grub.cfg` to 10.
Select kvos-test on boot to debug the test.

### Debugging Userspace

KVOS prints out the initexec binary file location. Run gdb as normal for debugging,
break on `switch_to_userspace`. Then run `add-symbol-file [initexec binary] [addr]`,
where addr is the location that KVOS maps the entrypoint of initexec to.

## Docs

Generate docs by running `cargo doc --workspace`

## Resoures

- We utilized the Philipp Oppermann tutorials [version 1](https://os.phil-opp.com/edition-1/) and [version 2](https://os.phil-opp.com/)
- We based some functionality (including ATA Port I/O) on [MOROS](https://github.com/vinc/moros)
- We utilized xv6 as a reference to understand how a Unix kernel is implemented : [xv6](https://github.com/mit-pdos/xv6-public/tree/master)

