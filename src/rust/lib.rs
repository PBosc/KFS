#![no_std]
#![no_main]

mod vga_buffer;
mod f16_shim;
mod port;
mod keyboard;
mod gdt;
mod stack;
mod shell;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
	gdt::init();
    vga_buffer::WRITER.lock().clear_screen();
	let mut shell = shell::Shell::new();
    shell.prompt();
    loop {
        if let Some(sc) = keyboard::poll_scancode() {
            match keyboard::KEYBOARD.lock().handle_scancode(sc) {
                keyboard::KeyAction::Char(c) => shell.on_char(c),
                keyboard::KeyAction::SwitchScreen(i) => {
                    vga_buffer::WRITER.lock().switch_screen(i);
					shell.on_switch(i);
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
