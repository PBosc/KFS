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

    pub fn reprompt(&self) {
        self.prompt();
        for &b in &self.buffer[..self.len] {
            print!("{}", b as char);
        }
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
                println!("commands: help, stack, halt, reboot, gdt");
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
            b"gdt" => {
                crate::gdt::read_gdtr();
            }
                // in shell dispatch:
            b"sig" => {
                crate::signal::SIGNALS.lock().schedule(1);
                println!("signal 1 scheduled");
            }
            b"syscall" => {
                // demo 1: getpid (syscall 1) — returns a value via eax
                let pid: u32;
                unsafe {
                    core::arch::asm!(
                        "int 0x80",
                        in("eax") 1u32,        // syscall 1 = getpid
                        lateout("eax") pid,    // return value comes back in eax
                    );
                }
                println!("getpid() = {}", pid);
            
                // demo 2: write (syscall 0) — passes args via registers, prints
                let msg = b"hello from int 0x80\n";
                unsafe {
                    core::arch::asm!(
                        "int 0x80",
                        in("eax") 0u32,        // syscall 0 = write
                        in("ebx") 1u32,        // fd (ignored for now)
                        in("ecx") msg.as_ptr() as u32,
                        in("edx") msg.len() as u32,
                        lateout("eax") _,      // clobbered return
                    );
                }
                // demo 3: unknown syscall -> returns -1 (0xFFFFFFFF)
                let ret: u32;
                unsafe {
                    core::arch::asm!(
                        "int 0x80",
                        in("eax") 99u32,       // no such syscall
                        lateout("eax") ret,
                    );
                }
                println!("syscall(99) = {:#x} (unknown -> -1)", ret);
            }
            b"azerty" => {
                crate::keyboard::KEYBOARD.lock().toggle_layout();
                println!("keyboard layout toggled");
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