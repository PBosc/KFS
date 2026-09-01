# kfs: A Minimal Rust Kernel for i386

A minimal operating system kernel written in Rust and assembly for the i386
architecture. It boots via the Multiboot2 specification using GRUB and provides
a VGA text-mode terminal with keyboard input, colors, and multiple virtual
screens.

## Features

### Base
- **Rust kernel** with `#![no_std]` / `#![no_main]`.
- **Assembly boot code** (NASM), Multiboot2-compliant, sets up the stack and
  zeroes `.bss` before calling into Rust.
- **Custom linker script** (`linker.ld`) defining the memory layout and entry point.
- **VGA text output** written directly to the `0xB8000` text buffer, exposed
  through Rust's formatting macros (`print!` / `println!`).
- **Bootable ISO** generated with GRUB.

### Bonus
- **Scrolling** text output.
- **Hardware cursor** tracking the write position (VGA CRTC).
- **Color support**, changeable at runtime (foreground and background).
- **PS/2 keyboard** (polled) with a US-QWERTY layout, handling Shift, Caps Lock,
  and Ctrl.
- **Three virtual screens**, each with independent content, cursor, and color,
  switchable via keyboard shortcuts.

## Controls

| Shortcut              | Action                          |
| --------------------- | ------------------------------- |
| Ctrl + F1 / F2 / F3   | Switch to screen 1 / 2 / 3      |
| Ctrl + 1..8           | Set foreground color            |
| Ctrl + Shift + 1..8   | Set background color            |

## Project Structure

```
kfs/
├── .cargo/
│   └── config.toml          # build-std / target flags
├── iso/
│   └── boot/
│       └── grub/
│           └── grub.cfg
├── src/
│   ├── asm/
│   │   ├── boot.asm         # stack setup, .bss zeroing, calls kernel_main
│   │   └── multiboot.asm    # Multiboot2 header
│   └── rust/
│       ├── lib.rs           # kernel entry point, main loop
│       ├── vga_buffer.rs    # VGA driver, screens, cursor, colors
│       ├── keyboard.rs      # PS/2 scancode handling, layout, modifiers
│       ├── port.rs          # inb / outb I/O port primitives
│       └── f16_shim.rs      # soft-float shim
├── Cargo.toml
├── i386-unknown-none.json   # custom bare-metal target
├── linker.ld
├── Makefile
├── rust-toolchain.toml      # pinned nightly + components
└── README.md
```

## Prerequisites

- **Rust nightly** (pinned in `rust-toolchain.toml`) with the `rust-src` and
  `llvm-tools-preview` components. The build recompiles `core`/`alloc` via
  `-Zbuild-std` against the custom `i386-unknown-none.json` target; the required
  flags live in `.cargo/config.toml`.
- **NASM** — assembles the boot code.
- **GRUB** (`grub-mkrescue`) and **xorriso** — build the bootable ISO.
- **KVM/QEMU** — runs the kernel.

On a machine without GRUB/xorriso, `make install_requirements` builds them from
source into `~/.local`.

## Building and Running

```sh
make            # build kernel.bin (release)
make iso        # package into kfs.iso
make run        # boot in KVM
make clean      # remove build artifacts
make fclean     # clean + cargo clean
make re         # fclean + build
```

### Debug vs Release

```sh
make DEBUG=1        # unoptimized build with debuginfo
make DEBUG=1 run    # rebuild in debug and boot
```

This switches the Rust profile and links the matching
`target/i386-unknown-none/<profile>/libkfs.a` into `kernel.bin`.

## How It Works

1. **GRUB** finds the Multiboot2 header in `multiboot.asm` and loads the kernel
   into protected mode.
2. **`boot.asm`** sets up a stack, zeroes `.bss`, and calls `kernel_main`.
3. **`kernel_main`** clears the screen and enters a loop polling the keyboard,
   printing typed characters and handling screen/color shortcuts.

## Acknowledgments

- Inspired by the [Writing an OS in Rust](https://os.phil-opp.com/) series.
- Uses GRUB for bootloading and KVM/QEMU for virtualization.