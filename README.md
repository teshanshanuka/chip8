# Chip-8 emulator

Chip-8 language emulator written in **Rust**. The implementation is done following the [chip8-book](https://github.com/aquova/chip8-book).

## Games

- [Chip-8 database](https://archive.org/details/chip-8-games)
- [In the book repo](https://github.com/aquova/chip8-book/tree/master/roms)
- [Chip-8 roms repo](https://github.com/kripod/chip8-roms)
- [Chip-8 C emulator repo](https://github.com/dmatlack/chip8)

## Usage

```sh
cd desktop
cargo run <path/to/rom>
```

### Web frontend

**Setup**

```sh
cargo install wasm-pack
cd wasm
wasm-pack build --target web
mv pkg/wasm_bg.wasm pkg/wasm.js ../web/
```

**Run http server for the web frontend**

```sh
cd web/
python3 -m http.server
```
