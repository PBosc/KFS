const _: () = assert!(core::mem::size_of::<GdtEntry>() == 8);

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct GdtEntry {
    limit_low: u16,   // limit[0:15]
    base_low: u16,    // base[0:15]
    base_mid: u8,     // base[16:23]
    access: u8,       // access byte
    flags_limit: u8,  // low nibble = limit[16:19], high nibble = flags
    base_high: u8,    // base[24:31]
}

impl GdtEntry {
    const fn new(base: u32, limit: u32, access: u8, flags: u8) -> GdtEntry {
        GdtEntry {
            limit_low:  (limit & 0xFFFF) as u16,
            base_low:   (base & 0xFFFF) as u16,
            base_mid:   ((base >> 16) & 0xFF) as u8,
            access,
            // low nibble: limit bits 16-19; high nibble: flags
            flags_limit: (((limit >> 16) & 0x0F) as u8) | ((flags & 0x0F) << 4),
            base_high:  ((base >> 24) & 0xFF) as u8,
        }
    }
}

#[repr(C, packed)]
struct GdtPtr {
    limit: u16,   // size of GDT in bytes, minus 1
    base: u32,    // linear address where the GDT lives (0x800)
}

static GDT: [GdtEntry; 7] = [
    GdtEntry::new(0, 0, 0, 0),                    // 0: null descriptor (required)
    GdtEntry::new(0, 0xFFFFF, 0x9A, 0xC),         // 1: kernel code  (ring 0, exec, read)
    GdtEntry::new(0, 0xFFFFF, 0x92, 0xC),         // 2: kernel data  (ring 0, data, write)
    GdtEntry::new(0, 0xFFFFF, 0x92, 0xC),         // 3: kernel stack (ring 0, data, write)
    GdtEntry::new(0, 0xFFFFF, 0xFA, 0xC),         // 4: user code    (ring 3, exec, read)
    GdtEntry::new(0, 0xFFFFF, 0xF2, 0xC),         // 5: user data    (ring 3, data, write)
    GdtEntry::new(0, 0xFFFFF, 0xF2, 0xC),         // 6: user stack   (ring 3, data, write)
];

const GDT_ADDR: u32 = 0x800;

pub fn init() {
    unsafe {
        // 1. copy the descriptors to physical 0x800
        let dst = GDT_ADDR as *mut GdtEntry;
        for (i, entry) in GDT.iter().enumerate() {
            core::ptr::write(dst.add(i), *entry);
        }

        // 2. build the GDTR pointer
        let gdt_ptr = GdtPtr {
            limit: (core::mem::size_of::<[GdtEntry; 7]>() - 1) as u16,
            base: GDT_ADDR,
        };

        // 3. load it and reload segments
        load_gdt(&gdt_ptr);
    }
}

unsafe fn load_gdt(ptr: &GdtPtr) {
    unsafe {
        core::arch::asm!(
            "lgdt [{ptr}]",
            "mov ax, 0x10",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",
            // far jump via far-return: push selector, push target, retf
            "push 0x08",
            "lea {tmp}, [2f]",
            "push {tmp}",
            "retf",
            "2:",
            ptr = in(reg) ptr,
            tmp = out(reg) _,
            out("ax") _,
        );
    }
}

pub fn read_gdtr() -> (u16, u32) {
    let mut ptr = GdtPtr { limit: 0, base: 0 };
    unsafe {
        core::arch::asm!("sgdt [{}]", in(reg) &mut ptr);
    }
    (ptr.limit, ptr.base)
}