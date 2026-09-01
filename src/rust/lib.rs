#![no_std]
#![no_main]

mod vga_buffer;
mod f16_shim;
mod port;
mod keyboard;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
	vga_buffer::WRITER.lock().clear_screen();

    loop {
        if let Some(sc) = keyboard::poll_scancode() {
            let action = keyboard::KEYBOARD.lock().handle_scancode(sc);
            // KEYBOARD lock released here (temporary dropped) BEFORE we take WRITER
            match action {
                keyboard::KeyAction::Char(c) => print!("{}", c),
                keyboard::KeyAction::SwitchScreen(i) => {
                    vga_buffer::WRITER.lock().switch_screen(i);
                }
                keyboard::KeyAction::SetForeground(c) => {
                    vga_buffer::WRITER.lock().set_foreground(vga_buffer::Color::from_u8(c));
                }
                keyboard::KeyAction::SetBackground(c) => {
                    vga_buffer::WRITER.lock().set_background(vga_buffer::Color::from_u8(c));
                }
                keyboard::KeyAction::None => {}
            }
        }
    }
}
