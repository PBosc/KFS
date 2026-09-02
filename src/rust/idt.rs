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

// no-error-code handlers
extern "x86-interrupt" fn divide_by_zero_handler(_frame: InterruptStackFrame) {
    panic_exception("DIVIDE BY ZERO");
}

extern "x86-interrupt" fn invalid_opcode_handler(_frame: InterruptStackFrame) {
    panic_exception("INVALID OPCODE");
}

extern "x86-interrupt" fn breakpoint_handler(_frame: InterruptStackFrame) {
    println!("EXCEPTION: breakpoint");
    // trap — CPU resumes after int3 automatically
}

// error-code handlers — note the extra u32 parameter
extern "x86-interrupt" fn general_protection_handler(_frame: InterruptStackFrame, error_code: u32) {
    println!("EXCEPTION: general protection fault, code={:#x}", error_code);
    hang();
}

extern "x86-interrupt" fn page_fault_handler(_frame: InterruptStackFrame, error_code: u32) {
    // read CR2 — the faulting address
    let cr2: u32;
    unsafe { asm!("mov {}, cr2", out(reg) cr2); }
    println!("EXCEPTION: page fault at {:#x}, code={:#x}", cr2, error_code);
    hang();
}

// double fault: error code always 0, and it must NOT return (diverging)
extern "x86-interrupt" fn double_fault_handler(_frame: InterruptStackFrame, _error_code: u32) -> ! {
    println!("EXCEPTION: DOUBLE FAULT");
    hang();
}

extern "x86-interrupt" fn keyboard_handler(_frame: InterruptStackFrame) {
    let scancode = unsafe { crate::port::inb(0x60) };
    crate::keyboard::KEYBOARD.lock().push_scancode(scancode);
    unsafe { crate::port::outb(0x20, 0x20); }   // EOI to master PIC
}

fn hang() -> ! {
    loop {
        unsafe { asm!("cli", "hlt"); }
    }
}

fn panic_exception(name: &str) -> ! {
    println!("EXCEPTION: {}", name);
    hang();
}

fn set_handler_err(vector: usize, handler: extern "x86-interrupt" fn(InterruptStackFrame, u32)) {
    let addr = handler as u32;
    unsafe {
        IDT[vector] = IdtEntry::new(addr, 0x8E);
    }
}

// double fault is diverging (-> !), needs its own setter signature
fn set_handler_diverging(vector: usize, handler: extern "x86-interrupt" fn(InterruptStackFrame, u32) -> !) {
    let addr = handler as u32;
    unsafe {
        IDT[vector] = IdtEntry::new(addr, 0x8E);
    }
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

    // faults
    set_handler(0, divide_by_zero_handler);
    set_handler(3, breakpoint_handler);
    set_handler(6, invalid_opcode_handler);
    set_handler_diverging(8, double_fault_handler);
    set_handler_err(13, general_protection_handler);
    set_handler_err(14, page_fault_handler);

    // interrupts
    set_handler(0x21, keyboard_handler);
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