use crate::print;
use crate::println;

const CMD_MAX: usize = 128;

pub struct Shell {
    buffer: [u8; CMD_MAX],
    len: usize,
    initialized: [bool; 3],
}

impl Shell {
    pub const fn new() -> Shell {
        Shell {
            buffer: [0; CMD_MAX],
            len: 0,
            initialized: [true, false, false],
        }
    }

    pub fn prompt(&self) {
        print!("> ");
    }

    // called for each character the keyboard produces
    pub fn on_char(&mut self, c: char) {
        match c {
            '\n' => {
                print!("\n");
                self.dispatch();
                self.len = 0;
                self.prompt();
            }
            '\x08' => self.backspace(),   // backspace (0x08)
            _ => {
                if self.len < CMD_MAX {
                    self.buffer[self.len] = c as u8;
                    self.len += 1;
                    print!("{}", c);       // echo
                }
            }
        }
    }

    fn backspace(&mut self) {
        if self.len > 0 {
            self.len -= 1;
            print!("\x08");
        }
    }

        fn dispatch(&mut self) {
        let line = &self.buffer[..self.len];
        match line {
            b"" => {}                          // empty line, do nothing
            b"help" => {
                println!("commands: help, stack, halt, reboot");
            }
            b"stack" => {
                crate::stack::print_stack(256);
            }
            b"halt" => {
                println!("halting");
                unsafe {
                    core::arch::asm!("cli", "hlt");
                }
            }
            b"reboot" => {
                println!("rebooting");
                unsafe {
                    // pulse the 8042 keyboard controller reset line
                    crate::port::outb(0x64, 0xFE);
					loop { core::arch::asm!("hlt"); }
                }
            }
            _ => {
                print!("unknown command: ");
                for &b in line { print!("{}", b as char); }
                println!();
            }
        }
    }
	
	pub fn on_switch(&mut self, screen: usize) {
        if !self.initialized[screen] {
            self.prompt();
            self.initialized[screen] = true;
        }
    }
}