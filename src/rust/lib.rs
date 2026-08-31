#![no_std]
#![no_main]

mod vga_buffer;
mod f16_shim;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    println!("Hello World{}", "!");

    loop {}
}
