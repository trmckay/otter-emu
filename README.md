# otter-emu

WIP emulator for Cal Poly's RISC-V RV32I chip written in Rust.
Run binaries compiled for the RV32I architecture.

## Preview

![Screenshot](res/img/screenshot.png)

## TODO
- [x] GUI
- [ ] CLI
- [ ] Interrupt support
- [x] Breakpoint support (backend)
- [ ] Breakpoint support (frontend)
- [ ] In-place register/memory editing
- [ ] Integrated cross-compiler

## Building

### Linux

- Rust (2018)
- GTK
- Glib

Other dependencies are provided by Cargo.

`cargo build --release`

### Other OS/architecture

See `gtk-rs` [documentation](https://gtk-rs.org/docs-src/tutorial/cross)
for targeting architectures other than 64-bit Linux.

On Windows, it may be easier to run a Linux binary in WSL2 (detailed instructions to come).
