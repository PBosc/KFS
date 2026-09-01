use lazy_static::lazy_static;
use spin::Mutex;

static SCANCODE_SET1: [u8; 128] = [
    0,    0x1B, b'1', b'2', b'3', b'4', b'5', b'6',   // 0x00-0x07  (0x01=Esc)
    b'7', b'8', b'9', b'0', b'-', b'=', 0x08, b'\t',  // 0x08-0x0F  (0x0E=Backspace)
    b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i',   // 0x10-0x17
    b'o', b'p', b'[', b']', b'\n', 0,   b'a', b's',   // 0x18-0x1F  (0x1C=Enter, 0x1D=LCtrl)
    b'd', b'f', b'g', b'h', b'j', b'k', b'l', b';',   // 0x20-0x27
    b'\'', b'`', 0,   b'\\', b'z', b'x', b'c', b'v',  // 0x28-0x2F  (0x2A=LShift)
    b'b', b'n', b'm', b',', b'.', b'/', 0,   b'*',    // 0x30-0x37  (0x36=RShift)
    0,    b' ', 0,   0,   0,   0,   0,   0,           // 0x38-0x3F  (0x38=LAlt, 0x39=Space, 0x3A=Caps)
    0,    0,    0,    0,    0,    0,    0,    0,       // 0x40-0x47  (F-keys)
    0,    0,    0,    0,    0,    0,    0,    0,       // 0x48-0x4F
    0,    0,    0,    0,    0,    0,    0,    0,       // 0x50-0x57
    0,    0,    0,    0,    0,    0,    0,    0,       // 0x58-0x5F
    0,    0,    0,    0,    0,    0,    0,    0,       // 0x60-0x67
    0,    0,    0,    0,    0,    0,    0,    0,       // 0x68-0x6F
    0,    0,    0,    0,    0,    0,    0,    0,       // 0x70-0x77
    0,    0,    0,    0,    0,    0,    0,    0,       // 0x78-0x7F
];

static SCANCODE_SET1_SHIFT: [u8; 128] = [
    0,    0x1B, b'!', b'@', b'#', b'$', b'%', b'^',   // 0x00-0x07
    b'&', b'*', b'(', b')', b'_', b'+', 0x08, b'\t',  // 0x08-0x0F
    b'Q', b'W', b'E', b'R', b'T', b'Y', b'U', b'I',   // 0x10-0x17
    b'O', b'P', b'{', b'}', b'\n', 0,   b'A', b'S',   // 0x18-0x1F
    b'D', b'F', b'G', b'H', b'J', b'K', b'L', b':',   // 0x20-0x27
    b'"', b'~', 0,   b'|', b'Z', b'X', b'C', b'V',    // 0x28-0x2F
    b'B', b'N', b'M', b'<', b'>', b'?', 0,   b'*',    // 0x30-0x37
    0,    b' ', 0,   0,   0,   0,   0,   0,           // 0x38-0x3F
    0,    0,    0,    0,    0,    0,    0,    0,       // 0x40-0x47  (F-keys)
    0,    0,    0,    0,    0,    0,    0,    0,       // 0x48-0x4F
    0,    0,    0,    0,    0,    0,    0,    0,       // 0x50-0x57
    0,    0,    0,    0,    0,    0,    0,    0,       // 0x58-0x5F
    0,    0,    0,    0,    0,    0,    0,    0,       // 0x60-0x67
    0,    0,    0,    0,    0,    0,    0,    0,       // 0x68-0x6F
    0,    0,    0,    0,    0,    0,    0,    0,       // 0x70-0x77
    0,    0,    0,    0,    0,    0,    0,    0,       // 0x78-0x7F
];

pub fn poll_scancode() -> Option<u8> {
    unsafe {
        let status = crate::port::inb(0x64);
        if status & 0x01 != 0 {
            Some(crate::port::inb(0x60))
        } else {
            None
        }
    }
}

pub enum KeyAction {
    Char(char),
    SwitchScreen(usize),
    SetForeground(u8),
    SetBackground(u8),
    None,
}

pub struct Keyboard {
    shift: bool,
    caps: bool,
    ctrl: bool,
}

impl Keyboard {
    pub const fn new() -> Keyboard {
        Keyboard { shift: false, caps: false, ctrl: false }
    }

    pub fn handle_scancode(&mut self, scancode: u8) -> KeyAction {
        match scancode {
            0x1D => { self.ctrl = true;  return KeyAction::None; } // ctrl press
            0x9D => { self.ctrl = false; return KeyAction::None; } // ctrl release
            0x2A | 0x36 => { self.shift = true;  return KeyAction::None; }
            0xAA | 0xB6 => { self.shift = false; return KeyAction::None; }
            0x3A        => { self.caps = !self.caps; return KeyAction::None; }
            _ => {}
        }

        if scancode & 0x80 != 0 {
            return KeyAction::None; // other releases
        }

        // shortcuts intercepted BEFORE char translation
        if self.ctrl {
            // screen switches (F-keys)
            match scancode {
                0x3B => return KeyAction::SwitchScreen(0),
                0x3C => return KeyAction::SwitchScreen(1),
                0x3D => return KeyAction::SwitchScreen(2),
                _ => {}
            }
            // number row 1..=8  →  scancodes 0x02..=0x09  →  color index 0..=7
            if scancode >= 0x02 && scancode <= 0x09 {
                let color = scancode - 0x02; // 0x02 -> 0, 0x03 -> 1, ...
                if self.shift {
                    return KeyAction::SetBackground(color);
                } else {
                    return KeyAction::SetForeground(color);
                }
            }
        }

        let base = SCANCODE_SET1[scancode as usize];
        if base == 0 {
            return KeyAction::None;
        }
        let ch = if self.should_uppercase(base) {
            SCANCODE_SET1_SHIFT[scancode as usize]
        } else {
            base
        };
        if ch == 0 { KeyAction::None } else { KeyAction::Char(ch as char) }
    }

    fn should_uppercase(&self, base: u8) -> bool {
        if base.is_ascii_alphabetic() {
            self.shift ^ self.caps
        } else {
            self.shift
        }
    }
}

lazy_static! {
	pub static ref KEYBOARD: Mutex<Keyboard> = Mutex::new(Keyboard::new());
}