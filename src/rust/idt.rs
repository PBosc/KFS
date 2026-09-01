use core::arch::asm;
use crate::println;
const _: () = assert!(core::mem::size_of::<IdtEntry>() == 8);

#[repr(C)]
pub struct InterruptStackFrame {
    pub eip: u32,
    pub cs: u32,
    pub eflags: u32,
    pub esp: u32,
    pub ss: u32,
}

// x86-interrupt ABI: the compiler generates the correct entry/iret.
// The InterruptStackFrame is pushed by the CPU; we take it by ref.
extern "x86-interrupt" fn breakpoint_handler(_frame: InterruptStackFrame) {
    println!("EXCEPTION: breakpoint");
    // breakpoint is a trap CPU resumes after the int3 automatically
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct IdtEntry {
    offset_low: u16,    // handler address bits 0-15
    selector: u16,      // code segment selector (0x08)
    zero: u8,           // always 0
    type_attr: u8,      // gate type + DPL + present
    offset_high: u16,   // handler address bits 16-31
}

const KERNEL_CODE_SELECTOR: u16 = 0x08;  // from GDT

impl IdtEntry {
    const fn new(handler: u32, type_attr: u8) -> IdtEntry {
        IdtEntry {
            offset_low: (handler & 0xFFFF) as u16,
            selector: KERNEL_CODE_SELECTOR,
            zero: 0,
            type_attr,
            offset_high: ((handler >> 16) & 0xFFFF) as u16,
        }
    }

    const fn missing() -> IdtEntry {
        // present bit clear -> unused vector
        IdtEntry::new(0, 0)
    }
}

const IDT_SIZE: usize = 256;

static mut IDT: [IdtEntry; IDT_SIZE] = [IdtEntry::missing(); IDT_SIZE];

#[repr(C, packed)]
struct IdtPtr {
    limit: u16,   // size in bytes minus 1
    base: u32,    // address of the IDT
}

fn set_handler(vector: usize, handler: extern "x86-interrupt" fn(InterruptStackFrame)) {
    let addr = handler as u32;
    unsafe {
        IDT[vector] = IdtEntry::new(addr, 0x8E);  // present, ring 0, interrupt gate
    }
}

pub fn init() {
    set_handler(3, breakpoint_handler);   // breakpoint
    unsafe {
        let idt_ptr = IdtPtr {
            limit: (core::mem::size_of::<[IdtEntry; IDT_SIZE]>() - 1) as u16,
            base: core::ptr::addr_of!(IDT) as u32,
        };
        load_idt(&idt_ptr);
    }
}

unsafe fn load_idt(ptr: &IdtPtr) {
    unsafe {
        core::arch::asm!("lidt [{}]", in(reg) ptr, options(readonly, nostack, preserves_flags));
    }
}