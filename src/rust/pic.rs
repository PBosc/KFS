use crate::port::outb;

const PIC1_CMD: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_CMD: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

const ICW1_INIT: u8 = 0x11;   // start init, expect ICW4
const ICW4_8086: u8 = 0x01;   // 8086/88 mode

pub fn remap() {
    unsafe {
        // ICW1: start initialization sequence on both PICs
        outb(PIC1_CMD, ICW1_INIT);
        outb(PIC2_CMD, ICW1_INIT);

        // ICW2: set the vector offset master IRQs -> 0x20, slave -> 0x28
        outb(PIC1_DATA, 0x20);   // master: IRQ0-7 -> vectors 0x20-0x27
        outb(PIC2_DATA, 0x28);   // slave:  IRQ8-15 -> vectors 0x28-0x2F

        // ICW3: tell master slave is on IRQ2 (0b100), tell slave its cascade id
        outb(PIC1_DATA, 0x04);   // master: slave at IRQ2
        outb(PIC2_DATA, 0x02);   // slave: cascade identity

        // ICW4: 8086 mode
        outb(PIC1_DATA, ICW4_8086);
        outb(PIC2_DATA, ICW4_8086);

        // mask all IRQs for now
        outb(PIC1_DATA, 0xFF);
        outb(PIC2_DATA, 0xFF);
    }
}

pub fn unmask_irq1() {
    unsafe {
        // read current mask, clear bit 1 (IRQ1), write back
        let mask = crate::port::inb(0x21);
        outb(0x21, mask & !0x02);   // bit 1 = IRQ1
    }
}