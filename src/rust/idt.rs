use core::arch::asm;
use crate::println;
use crate::print;
const _: () = assert!(core::mem::size_of::<IdtEntry>() == 8);

#[repr(C)]
pub struct InterruptStackFrame {
    pub eip: u32,
    pub cs: u32,
    pub eflags: u32,
    pub esp: u32,
    pub ss: u32,
}

#[repr(C)]
pub struct Registers {
    // pushed by pusha, in the order they sit in memory (low to high)
    pub edi: u32,
    pub esi: u32,
    pub ebp: u32,
    pub esp_dummy: u32,   // pusha pushes esp but it's the pre-pusha value; not useful
    pub ebx: u32,
    pub edx: u32,
    pub ecx: u32,
    pub eax: u32,
    // pushed by our stub
    pub int_no: u32,
    pub err_code: u32,
    // pushed by the CPU
    pub eip: u32,
    pub cs: u32,
    pub eflags: u32,
}

// macro for no-error-code ISR stubs
macro_rules! isr_no_err {
    ($stub:ident, $int_no:expr, $inner:ident) => {
        #[unsafe(naked)]
        extern "C" fn $stub() {
            core::arch::naked_asm!(
                "push 0",              // dummy error code
                concat!("push ", stringify!($int_no)),
                "pusha",
                "push esp",
                "call {inner}",
                "add esp, 4",
                "popa",
                "add esp, 8",
                "iretd",
                inner = sym $inner,
            );
        }
    };
}

// macro for error-code ISR stubs (CPU already pushed the error code)
macro_rules! isr_err {
    ($stub:ident, $int_no:expr, $inner:ident) => {
        #[unsafe(naked)]
        extern "C" fn $stub() {
            core::arch::naked_asm!(
                // NO dummy push — CPU already pushed the real error code
                concat!("push ", stringify!($int_no)),
                "pusha",
                "push esp",
                "call {inner}",
                "add esp, 4",
                "popa",
                "add esp, 8",          // removes int_no + error_code
                "iretd",
                inner = sym $inner,
            );
        }
    };
}

extern "C" fn general_protection_inner(regs: *const Registers) {
    let r = unsafe { &*regs };
    kernel_panic_regs("GENERAL PROTECTION FAULT", r);
}
extern "C" fn page_fault_inner(regs: *const Registers) {
    let r = unsafe { &*regs };
    let cr2: u32;
    unsafe { asm!("mov {}, cr2", out(reg) cr2); }
    println!("page fault at address {:#x}", cr2);
    kernel_panic_regs("PAGE FAULT", r);
}
extern "C" fn divide_by_zero_inner(regs: *const Registers) {
    let r = unsafe { &*regs };
    kernel_panic_regs("DIVIDE BY ZERO", r);
}
extern "C" fn invalid_opcode_inner(regs: *const Registers) {
    let r = unsafe { &*regs };
    kernel_panic_regs("INVALID OPCODE", r);
}
extern "C" fn double_fault_inner(regs: *const Registers) {
    let r = unsafe { &*regs };
    kernel_panic_regs("DOUBLE FAULT", r);
}

extern "C" fn breakpoint_inner(_regs: *const Registers) {
    // do nothing — just test the stub entry/exit path
}

#[unsafe(naked)]
extern "C" fn isr_keyboard() {
    unsafe {
        core::arch::naked_asm!(
            "push 0",           // dummy error code (keyboard has none)
            "push 0x21",        // interrupt number
            "pusha",            // push all GP registers
            "push esp",         // pass pointer to the frame as arg
            "call {handler}",   // call the Rust handler
            "add esp, 4",       // pop the arg
            "popa",             // restore GP registers
            "add esp, 8",       // pop int_no + err_code
            "iretd",            // return from interrupt
            handler = sym keyboard_handler_inner,
        );
    }
}

extern "C" fn keyboard_handler_inner(regs: *const Registers) {
    // now you have the full register frame
    let scancode = unsafe { crate::port::inb(0x60) };
    crate::keyboard::KEYBOARD.lock().push_scancode(scancode);
    unsafe { crate::port::outb(0x20, 0x20); }
    // regs available if you want them, e.g. for panic dumps
}

#[unsafe(naked)]
extern "C" fn isr_syscall() {
    core::arch::naked_asm!(
        "push 0",              // dummy error code
        "push 0x80",           // int number
        "pusha",
        "push esp",            // pointer to frame
        "call {inner}",
        "add esp, 4",
        "popa",                // restores eax (now holds return value) to caller
        "add esp, 8",
        "iretd",
        inner = sym syscall_inner,
    );
}

extern "C" fn syscall_inner(regs: *mut Registers) {
    let r = unsafe { &mut *regs };
    let syscall_num = r.eax;
    let arg1 = r.ebx;
    let arg2 = r.ecx;
    let arg3 = r.edx;

    let ret: u32 = match syscall_num {
        0 => sys_write(arg1, arg2, arg3),   // e.g. write
        1 => sys_getpid(),                   // stub — returns fake pid
        _ => 0xFFFFFFFF,                     // unknown syscall -> -1
    };

    r.eax = ret;   // return value goes back via eax after popa
}

fn sys_write(_fd: u32, ptr: u32, len: u32) -> u32 {
    // dumbest possible: treat ptr as a byte string, print len bytes
    let bytes = unsafe { core::slice::from_raw_parts(ptr as *const u8, len as usize) };
    for &b in bytes {
        print!("{}", b as char);
    }
    len   // return bytes written
}

fn sys_getpid() -> u32 {
    42   // no processes yet, return a placeholder
}

isr_no_err!(isr_divide_by_zero, 0, divide_by_zero_inner);
isr_no_err!(isr_breakpoint, 3, breakpoint_inner);
isr_no_err!(isr_invalid_opcode, 6, invalid_opcode_inner);
isr_err!(isr_double_fault, 8, double_fault_inner);
isr_err!(isr_general_protection, 13, general_protection_inner);
isr_err!(isr_page_fault, 14, page_fault_inner);

fn hang() -> ! {
    loop {
        unsafe { asm!("cli", "hlt"); }
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

fn set_handler_naked(vector: usize, handler: extern "C" fn()) {
    let addr = handler as u32;
    unsafe {
        IDT[vector] = IdtEntry::new(addr, 0x8E);
    }
}

pub fn init() {

    set_handler_naked(0, isr_divide_by_zero);
    set_handler_naked(3, isr_breakpoint);
    set_handler_naked(6, isr_invalid_opcode);
    set_handler_naked(8, isr_double_fault);
    set_handler_naked(13, isr_general_protection);
    set_handler_naked(14, isr_page_fault);
    set_handler_naked(0x21, isr_keyboard);
    set_handler_naked(0x80, isr_syscall);
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

pub fn kernel_panic(msg: &str) -> ! {
    unsafe { asm!("cli"); }   // no interrupts during the dump
    println!();
    println!("=== KERNEL PANIC ===");
    println!("{}", msg);
    hang();
}

// register-dumping variant, called from exception inners that have the frame
pub fn kernel_panic_regs(msg: &str, regs: &Registers) -> ! {
    unsafe { asm!("cli"); }
    println!();
    println!("=== KERNEL PANIC ===");
    println!("{}", msg);
    println!("registers:");
    println!("  eax={:#010x} ebx={:#010x} ecx={:#010x} edx={:#010x}",
             regs.eax, regs.ebx, regs.ecx, regs.edx);
    println!("  esi={:#010x} edi={:#010x} ebp={:#010x}",
             regs.esi, regs.edi, regs.ebp);
    println!("  eip={:#010x} cs={:#06x} eflags={:#010x}",
             regs.eip, regs.cs, regs.eflags);
    println!("  int_no={} err_code={:#x}", regs.int_no, regs.err_code);
    crate::stack::print_stack(256);
    println!("=== halted ===");
    hang();
}