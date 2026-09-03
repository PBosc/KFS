use lazy_static::lazy_static;
use spin::Mutex;

const QUEUE_SIZE: usize = 256;

pub enum KeyAction {
    Char(char),
    SwitchScreen(usize),
    SetForeground(u8),
    SetBackground(u8),
    None,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Layout {
    Qwerty,
    Azerty,
}

pub struct Keyboard {
    shift: bool,
    caps: bool,
    ctrl: bool,
    layout: Layout,
    queue: [u8; QUEUE_SIZE],
    head: usize,
    tail: usize,
}

impl Keyboard {
    pub const fn new() -> Keyboard {
        Keyboard {
            shift: false,
            caps: false,
            ctrl: false,
            layout: Layout::Qwerty,
            queue: [0; QUEUE_SIZE],
            head: 0,
            tail: 0,
        }
    }

    pub fn toggle_layout(&mut self) {
        self.layout = match self.layout {
            Layout::Qwerty => Layout::Azerty,
            Layout::Azerty => Layout::Qwerty,
        };
    }

    pub fn push_scancode(&mut self, sc: u8) {
        let next = (self.head + 1) % QUEUE_SIZE;
        if next != self.tail {          // buffer full -> drop, don't overwrite tail
            self.queue[self.head] = sc;
            self.head = next;
        }
    }

    pub fn pop_scancode(&mut self) -> Option<u8> {
        if self.head == self.tail {
            None
        } else {
            let sc = self.queue[self.tail];
            self.tail = (self.tail + 1) % QUEUE_SIZE;
            Some(sc)
        }
    }


    pub fn handle_scancode(&mut self, scancode: u8) -> KeyAction {
        match scancode {
            0x1D => { self.ctrl = true;  return KeyAction::None; }
            0x9D => { self.ctrl = false; return KeyAction::None; }
            0x2A | 0x36 => { self.shift = true;  return KeyAction::None; }
            0xAA | 0xB6 => { self.shift = false; return KeyAction::None; }
            0x3A        => { self.caps = !self.caps; return KeyAction::None; }
            _ => {}
        }

        if scancode & 0x80 != 0 {
            return KeyAction::None;
        }

        if self.ctrl {
            match scancode {
                0x3B => return KeyAction::SwitchScreen(0), // Ctrl+F1
                0x3C => return KeyAction::SwitchScreen(1), // Ctrl+F2
                0x3D => return KeyAction::SwitchScreen(2), // Ctrl+F3
                _ => {}
            }
            // number row 1..=8 -> scancodes 0x02..=0x09 -> color index 0..=7
            if scancode >= 0x02 && scancode <= 0x09 {
                let color = scancode - 0x02;
                if self.shift {
                    return KeyAction::SetBackground(color);
                } else {
                    return KeyAction::SetForeground(color);
                }
            }
        }
        let (base_table, shift_table): (&[u8; 128], &[u8; 128]) = match self.layout {
            Layout::Qwerty => (&SCANCODE_SET1, &SCANCODE_SET1_SHIFT),
            Layout::Azerty => (&SCANCODE_AZERTY, &SCANCODE_AZERTY_SHIFT),
        };
        let base = base_table[scancode as usize];
        if base == 0 { return KeyAction::None; }
        let ch = if self.should_uppercase(base) {
            shift_table[scancode as usize]
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


static SCANCODE_SET1: [u8; 128] = [
    0,    0x1B, b'1', b'2', b'3', b'4', b'5', b'6',
    b'7', b'8', b'9', b'0', b'-', b'=', 0x08, b'\t',
    b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i',
    b'o', b'p', b'[', b']', b'\n', 0,   b'a', b's',
    b'd', b'f', b'g', b'h', b'j', b'k', b'l', b';',
    b'\'', b'`', 0,   b'\\', b'z', b'x', b'c', b'v',
    b'b', b'n', b'm', b',', b'.', b'/', 0,   b'*',
    0   , b' ', 0,   0,0,0,0,0,
    0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,
];

static SCANCODE_SET1_SHIFT: [u8; 128] = [
    0,    0x1B, b'!', b'@', b'#', b'$', b'%', b'^',
    b'&', b'*', b'(', b')', b'_', b'+', 0x08, b'\t',
    b'Q', b'W', b'E', b'R', b'T', b'Y', b'U', b'I',
    b'O', b'P', b'{', b'}', b'\n', 0,   b'A', b'S',
    b'D', b'F', b'G', b'H', b'J', b'K', b'L', b':',
    b'"', b'~', 0,   b'|', b'Z', b'X', b'C', b'V',
    b'B', b'N', b'M', b'<', b'>', b'?', 0,   b'*',
    0,    b' ', 0,   0,   0,   0,   0,   0,
    0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,
];

// AZERTY base (unshifted). Number row gives symbols; digits need shift.
static SCANCODE_AZERTY: [u8; 128] = [
    0,    0x1B, b'&', b'\'', b'"', b'(', b'-', b'e',   // 0x00-07: 1..6 -> & é " ( - è  (é/è as ASCII stand-ins)
    b'_', b'c', b'a', b')', b'=', 0,    0x08, b'\t',   // 0x08-0F
    b'a', b'z', b'e', b'r', b't', b'y', b'u', b'i',    // 0x10-17: azerty top row (A Z E R T Y U I)
    b'o', b'p', b'^', b'$', b'\n', 0,   b'q', b's',    // 0x18-1F: ... O P, then Q S (A-row shifted)
    b'd', b'f', b'g', b'h', b'j', b'k', b'l', b'm',    // 0x20-27: D F G H J K L M
    0,    0,    0,    b'*', b'w', b'x', b'c', b'v',    // 0x28-2F: ... W X C V (Z-row)
    b'b', b'n', b',', b';', b':', b'!', 0,   b'*',     // 0x30-37
    0,    b' ', 0,   0,0,0,0,0,
    0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,
];

static SCANCODE_AZERTY_SHIFT: [u8; 128] = [
    0,    0x1B, b'1', b'2', b'3', b'4', b'5', b'6',    // shift on number row -> actual digits
    b'7', b'8', b'9', b'0', b'~', b'+', 0x08, b'\t',
    b'A', b'Z', b'E', b'R', b'T', b'Y', b'U', b'I',
    b'O', b'P', 0,   0,   b'\n', 0,   b'Q', b'S',
    b'D', b'F', b'G', b'H', b'J', b'K', b'L', b'M',
    b'%', 0,    0,   b'*', b'W', b'X', b'C', b'V',
    b'B', b'N', b'?', b'.', b'/', 0,   0,   b'*',
    0,    b' ', 0,   0,0,0,0,0,
    0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,
];