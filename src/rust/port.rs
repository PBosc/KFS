use core::arch::asm;

/// Write `value` to I/O port `port`.
///
/// # Safety
/// Arbitrary port writes can affect any hardware. Caller must know the port.
pub unsafe fn outb(port: u16, value: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") value,
            options(nomem, nostack, preserves_flags),
        );
    }
}

/// Read a byte from I/O port `port`.
///
/// # Safety
/// Arbitrary port reads can have side effects on some hardware.
pub unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    unsafe {
        asm!(
            "in al, dx",
            out("al") value,
            in("dx") port,
            options(nomem, nostack, preserves_flags),
        );
    }
    value
}