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
mod signal;

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
    // register a test handler for signal 1
    signal::SIGNALS.lock().register(1, || {
        println!("signal 1 handler ran!");
    });

    loop {
        // drain ALL pending scancodes, not just one
        loop {
            unsafe { core::arch::asm!("cli") };
            let sc = keyboard::KEYBOARD.lock().pop_scancode();
            unsafe { core::arch::asm!("sti") };
            match sc {
                Some(sc) => {
                    let action = keyboard::KEYBOARD.lock().handle_scancode(sc);
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
                }
                None => break,   // queue empty, stop draining
            }
        }
        // deliver pending signals
        let pending = signal::SIGNALS.lock().take_pending();
        let mut ran_any = false;
        for handler in pending.iter().flatten() {
            handler();
            ran_any = true;
        }
        if ran_any {
            shell.reprompt();   // reprint "> " + current buffer contents
        }

        unsafe { core::arch::asm!("hlt"); }
    }
}
