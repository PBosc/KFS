# kfs: A Minimal Rust Kernel for i386

A minimal operating system kernel written in Rust and assembly for the i386
architecture. It boots via the Multiboot2 specification using GRUB and provides
a VGA text-mode terminal with keyboard input, colors, multiple virtual screens,
a Global Descriptor Table, interrupt handling (IDT, PIC, exceptions), a signal
system, syscalls, and a small debug shell.

## Features

### Base
- **Rust kernel** with `#![no_std]` / `#![no_main]`.
- **Assembly boot code** (NASM), Multiboot2-compliant, sets up the stack and
  zeroes `.bss` before calling into Rust.
- **Custom linker script** (`linker.ld`) defining the memory layout and entry point.
- **VGA text output** written directly to the `0xB8000` text buffer, exposed
  through Rust's formatting macros (`print!` / `println!`).
- **Bootable ISO** generated with GRUB.

### KFS_2 — GDT & Stack
- **Global Descriptor Table** with six flat segments (kernel code/data/stack,
  user code/data/stack) plus the null descriptor, placed at physical address
  `0x00000800` and loaded via `lgdt`.
- **Segment reload** after loading the GDT, including a far return (`push`/`retf`)
  to reload `CS`.
- **Stack-dump tool** printing kernel stack memory as a human-readable hex dump.

### KFS_4 — Interrupts
- **Interrupt Descriptor Table** created, filled, and registered via `lidt`.
- **CPU exception handlers** (divide-by-zero, invalid opcode, double fault,
  general protection fault, page fault) via naked assembly stubs that capture the
  full register frame; both error-code and no-error-code exception shapes.
- **8259 PIC remap** moving hardware IRQs clear of the CPU exception vectors.
- **Interrupt-driven keyboard** on IRQ1, replacing the earlier polling loop; the
  handler enqueues scancodes and the main loop drains and processes them.
- **Kernel panic** dumping fault-time registers and the stack before halting.
- **Signal-callback system** — register callbacks by number, raise them
  immediately, or schedule them for deferred delivery from the main loop.

### Bonus
- **Scrolling** text output.
- **Hardware cursor** tracking the write position (VGA CRTC).
- **Color support**, changeable at runtime (foreground and background).
- **PS/2 keyboard** with US-QWERTY and French AZERTY layouts, handling Shift,
  Caps Lock, Ctrl, and backspace.
- **Three virtual screens**, each with independent content, cursor, and color.
- **Debug shell** with line editing and several commands.
- **Syscalls** via `int 0x80` — a dispatch table reading the syscall number and
  arguments from registers and returning a value in `eax`.

## Controls

| Shortcut              | Action                          |
| --------------------- | ------------------------------- |
| Ctrl + F1 / F2 / F3   | Switch to screen 1 / 2 / 3      |
| Ctrl + 1..8           | Set foreground color            |
| Ctrl + Shift + 1..8   | Set background color            |

## Shell commands

| Command   | Action                                        |
| --------- | --------------------------------------------- |
| `help`    | List available commands                       |
| `stack`   | Print a hex dump of the kernel stack          |
| `halt`    | Disable interrupts and halt the CPU           |
| `reboot`  | Reboot via the 8042 keyboard controller       |
| `syscall` | Trigger example syscalls via `int 0x80`       |
| `azerty`  | Toggle between QWERTY and AZERTY layouts       |

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
│       ├── keyboard.rs      # PS/2 scancodes, layouts, modifiers, ring buffer
│       ├── port.rs          # inb / outb I/O port primitives
│       ├── gdt.rs           # Global Descriptor Table
│       ├── idt.rs           # IDT, ISR stubs, exception & syscall handlers
│       ├── pic.rs           # 8259 PIC remap and IRQ masking
│       ├── signal.rs        # signal-callback system
│       ├── stack.rs         # kernel stack hex-dump tool
│       ├── shell.rs         # minimal debug shell
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
  flags live in `.cargo/config.toml`. Features used: `abi_x86_interrupt`,
  `naked_functions`.
- **NASM** — assembles the boot code.
- **GRUB** (`grub-mkrescue`) and **xorriso** — build the bootable ISO.
- **KVM/QEMU** — runs the kernel.

GRUB's build (if compiled from source) needs `bison`, `flex`, and `gettext`.

## Building and Running

```sh
make            # build kernel.bin (release)
make iso        # package into kfs.iso
make run        # boot in KVM/QEMU
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
`target/i386-unknown-none/<profile>/libkfs.a` into `kernel.bin`. The `dev`
profile disables overflow and debug assertions, which otherwise route into the
panic path and fault in this freestanding environment.

## How It Works

1. **GRUB** finds the Multiboot2 header in `multiboot.asm` and loads the kernel
   into protected mode.
2. **`boot.asm`** sets up a stack, zeroes `.bss`, and calls `kernel_main`.
3. **`kernel_main`** installs the GDT at `0x800`, sets up the IDT, remaps the PIC,
   enables interrupts, clears the screen, and enters a loop that drains keyboard
   input and delivers pending signals.
4. Interrupts (CPU exceptions, the keyboard IRQ, and `int 0x80` syscalls) are all
   dispatched through the IDT.

## Acknowledgments

- Inspired by the [Writing an OS in Rust](https://os.phil-opp.com/) series.
- Uses GRUB for bootloading and KVM/QEMU for virtualization.