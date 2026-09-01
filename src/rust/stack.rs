use crate::print;
use crate::println;

fn read_esp() -> u32 {
    let esp: u32;
    unsafe {
        core::arch::asm!("mov {}, esp", out(reg) esp);
    }
    esp
}

pub fn print_stack(bytes: usize) {
    let esp = read_esp();
    let start = esp as *const u8;
    let rows = bytes / 16;
    for row in 0..rows {
        let addr = esp as usize + row * 16;
        print!("{:08x}: ", addr);
        // hex bytes
        for col in 0..16 {
            let b = unsafe { core::ptr::read(start.add(row * 16 + col)) };
            print!("{:02x} ", b);
        }
        // ascii
        print!(" |");
        for col in 0..16 {
            let b = unsafe { core::ptr::read(start.add(row * 16 + col)) };
            let c = if b >= 32 && b <= 126 { b as char } else { '.' };
            print!("{}", c);
        }
        println!("|");
    }
}

