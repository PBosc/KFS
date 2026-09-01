use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;
const SCREEN_COUNT: usize = 3;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

impl ColorCode {
    fn foreground(&self) -> Color {
        Color::from_u8(self.0 & 0x0F)
    }
    fn background(&self) -> Color {
        Color::from_u8((self.0 >> 4) & 0x0F)
    }
}

impl Color {
    pub fn from_u8(v: u8) -> Color {
        match v {
            0 => Color::Black, 1 => Color::Blue, 2 => Color::Green, 3 => Color::Cyan,
            4 => Color::Red, 5 => Color::Magenta, 6 => Color::Brown, 7 => Color::LightGray,
            8 => Color::DarkGray, 9 => Color::LightBlue, 10 => Color::LightGreen, 11 => Color::LightCyan,
            12 => Color::LightRed, 13 => Color::Pink, 14 => Color::Yellow, _ => Color::White,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

#[derive(Clone, Copy)]
struct Screen {
    buffer: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
    row_position: usize,
    column_position: usize,
    color_code: ColorCode,
}

impl Screen {
    const fn new() -> Screen {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: ColorCode::new(Color::LightGray, Color::Black),
        };
        Screen {
            buffer: [[blank; BUFFER_WIDTH]; BUFFER_HEIGHT],
            row_position: 0,
            column_position: 0,
            color_code: ColorCode::new(Color::LightGray, Color::Black),
        }
    }
}

pub struct Writer {
    screens: [Screen; SCREEN_COUNT],
    active: usize,
    vga: &'static mut Buffer,
}

impl Writer {
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                32..=126 | b'\n' | 0x08 => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
			0x08 => self.backspace(),
            byte => {
                if self.screens[self.active].column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let active = self.active;
                let row = self.screens[active].row_position;
                let col = self.screens[active].column_position;
                let color = self.screens[active].color_code;
                let sc = ScreenChar { ascii_character: byte, color_code: color };
                self.screens[active].buffer[row][col] = sc;  // RAM (source of truth)
                self.vga.chars[row][col].write(sc);          // hardware mirror (volatile)
                self.screens[active].column_position += 1;
            }
        }
        self.update_cursor();
    }

    fn new_line(&mut self) {
        let active = self.active;
        if self.screens[active].row_position < BUFFER_HEIGHT - 1 {
            self.screens[active].row_position += 1;
        } else {
            // scroll RAM up by one row
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    self.screens[active].buffer[row - 1][col] =
                        self.screens[active].buffer[row][col];
                }
            }
            let blank = ScreenChar {
                ascii_character: b' ',
                color_code: self.screens[active].color_code,
            };
            for col in 0..BUFFER_WIDTH {
                self.screens[active].buffer[BUFFER_HEIGHT - 1][col] = blank;
            }
            self.flush(); // whole active screen -> hardware
        }
        self.screens[active].column_position = 0;
    }
	
	fn backspace(&mut self) {
        let active = self.active;
        if self.screens[active].column_position > 0 {
            self.screens[active].column_position -= 1;
            let row = self.screens[active].row_position;
            let col = self.screens[active].column_position;
            let color = self.screens[active].color_code;
            let blank = ScreenChar { ascii_character: b' ', color_code: color };
            self.screens[active].buffer[row][col] = blank;
            self.vga.chars[row][col].write(blank);
        }
    }

    pub fn clear_screen(&mut self) {
        let active = self.active;
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.screens[active].color_code,
        };
        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                self.screens[active].buffer[row][col] = blank;
            }
        }
        self.screens[active].row_position = 0;
        self.screens[active].column_position = 0;
        self.flush();
        self.update_cursor();
    }

    pub fn set_foreground(&mut self, foreground: Color) {
        let active = self.active;
        let bg = self.screens[active].color_code.background();
        self.screens[active].color_code = ColorCode::new(foreground, bg);
    }

    pub fn set_background(&mut self, background: Color) {
        let active = self.active;
        let fg = self.screens[active].color_code.foreground();
        self.screens[active].color_code = ColorCode::new(fg, background);
    }

    pub fn switch_screen(&mut self, index: usize) {
        if index >= SCREEN_COUNT || index == self.active {
            return;
        }
        self.active = index;
        self.flush();          // paint the newly-active screen's RAM buffer to hardware
        self.update_cursor();  // restore its saved cursor position
    }

    fn flush(&mut self) {
        let active = self.active;
        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let sc = self.screens[active].buffer[row][col];
                self.vga.chars[row][col].write(sc);
            }
        }
    }

    fn update_cursor(&self) {
        let s = &self.screens[self.active];
        let pos = s.row_position * BUFFER_WIDTH + s.column_position;
        unsafe {
            crate::port::outb(0x3D4, 0x0F);
            crate::port::outb(0x3D5, (pos & 0xFF) as u8);
            crate::port::outb(0x3D4, 0x0E);
            crate::port::outb(0x3D5, ((pos >> 8) & 0xFF) as u8);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        screens: [Screen::new(); SCREEN_COUNT],
        active: 0,
        vga: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}