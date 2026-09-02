#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(naked_functions)]

mod vga_buffer;
mod f16_shim;
mod port;
mod keyboard;
mod gdt;
mod stack;
mod shell;
mod idt;
mod pic;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
	gdt::init();
	idt::init();
    pic::remap();
    pic::unmask_irq1();
    vga_buffer::WRITER.lock().clear_screen();
    unsafe { core::arch::asm!("sti"); }
	let mut shell = shell::Shell::new();
    shell.prompt();
    loop {
        unsafe { core::arch::asm!("cli") };
        let action = {
            let mut kb = keyboard::KEYBOARD.lock();
            match kb.pop_scancode() {
                Some(sc) => kb.handle_scancode(sc),
                None => keyboard::KeyAction::None,
            }
        };  // lock dropped here
        unsafe { core::arch::asm!("sti") };

        // process OUTSIDE the lock and OUTSIDE cli — safe to take WRITER now
        match action {
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

        unsafe { core::arch::asm!("hlt"); }
    }
}
