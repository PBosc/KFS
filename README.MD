# kfs: A Minimal Rust Kernel for i386

This project is a minimal operating system kernel written in Rust and assembly for the i386 architecture. It demonstrates how to build a simple kernel that boots using the Multiboot2 specification and displays a message on the VGA text buffer.

## Features

- **Rust-based kernel**: The kernel is written in Rust with `#![no_std]` and `#![no_main]` attributes.
- **Assembly bootloader**: The bootloader is implemented in NASM assembly and adheres to the Multiboot2 specification.
- **Custom linker script**: A custom `linker.ld` script is used to define memory layout and entry points.
- **VGA text output**: The kernel writes directly to the VGA text buffer to display output.
- **ISO generation**: The kernel is packaged into a bootable ISO using GRUB.

## Project Structure

```
kfs/
├── .gitignore
├── Cargo.lock
├── Cargo.toml
├── i386-unknown-none.json
├── linker.ld
├── Makefile
├── README.MD
├── rust-toolchain.toml
├── .cargo/
│   └── config.toml
├── iso/
│   └── boot/
│       └── grub/
│           └── grub.cfg
└── src/
    ├── asm/
    │   ├── boot.asm
    │   └── multiboot.asm
    └── rust/
        └── lib.rs
```

- **`src/rust/lib.rs`**: The Rust kernel entry point.
- **`src/asm/boot.asm`**: The assembly bootloader that calls the kernel.
- **`src/asm/multiboot.asm`**: Multiboot2 header for GRUB compatibility.
- **`linker.ld`**: Defines the memory layout and entry point.
- **`Makefile`**: Automates the build, clean, and run processes.
- **`iso/boot/grub/grub.cfg`**: GRUB configuration for booting the kernel.

## Prerequisites

- **Rust nightly toolchain**: Install using `rustup` and ensure the `rust-toolchain.toml` is respected.
- **NASM**: For assembling the bootloader.
- **QEMU**: For running the kernel in a virtualized environment.

## Building and Running

1. **Build the kernel**:

   ```sh
   make all
   ```

2. **Create a bootable ISO**:

   ```sh
   make iso
   ```

3. **Run the kernel**:

   ```sh
   make run
   ```

4. **Clean the build artifacts**:

   ```sh
   make clean
   ```

5. **Fully clean the project**:
   ```sh
   make fclean
   ```

## Debug vs Release builds

The `Makefile` supports toggling between release and debug profiles using the `DEBUG` variable.

- Default (release): optimized build
   ```sh
   make                 # builds with `cargo build --release`
   make iso             # packages current build into an ISO
   ```
- Debug: unoptimized with debuginfo
   ```sh
   make DEBUG=1         # builds with `cargo build`
   make DEBUG=1 iso     # ensures the kernel is rebuilt in debug before creating the ISO
   ```

Under the hood, this switches the Rust build profile and links the appropriate
`target/i386-unknown-none/<profile>/libkfs.a` into `kernel.bin`.

## How It Works

1. **Bootloader**: The `boot.asm` file initializes the system and calls the `kernel_main` function in Rust.
2. **Kernel**: The `kernel_main` function clears the VGA buffer and writes "42" in the center of the screen.
3. **GRUB**: The Multiboot2 header in `multiboot.asm` ensures compatibility with GRUB, which loads the kernel.

## Acknowledgments

- Inspired by the [Writing an OS in Rust](https://os.phil-opp.com/) series.
- Uses GRUB for bootloading and QEMU for virtualization.
