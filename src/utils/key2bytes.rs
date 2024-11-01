use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
#[derive(Clone)]
pub enum U8Code {
    Ascii(u8),
    TriU8([u8; 3]),
    Auto(Vec<u8>),
}
pub trait ToBytes {
    fn into_byte_code(self) -> U8Code;
}
impl ToBytes for KeyEvent {
    fn into_byte_code(self) -> U8Code {
        let KeyEvent {
            code, modifiers, ..
        } = self;
        if modifiers.contains(KeyModifiers::CONTROL) {
            if let KeyCode::Char(ch) = code {
                U8Code::Ascii(ascii::ctrl(ch).expect("Not in a..=z or A..=Z or 0..=9 or + or -"))
            } else {
                unimplemented!("Not in a..=z or A..=Z or 0..=9 or + or -")
            }
        } else if modifiers.contains(KeyModifiers::ALT) {
            if let KeyCode::Char(ch) = code {
                U8Code::TriU8(ascii::alt(ch).expect("Not in a..=z or A..=Z or 0..=9"))
            } else {
                unimplemented!("Not in a..=z or A..=Z or 0..=9")
            }
        } else {
            code.into_byte_code()
        }
    }
}
impl ToBytes for KeyCode {
    fn into_byte_code(self) -> U8Code {
        match self {
            KeyCode::Char(input) => U8Code::Auto(input.to_string().into_bytes()),
            KeyCode::Backspace => U8Code::Ascii(ascii::BACKSPACE),
            KeyCode::Enter => U8Code::Ascii(ascii::ENTER),
            KeyCode::Left => U8Code::TriU8(LEFT),
            KeyCode::Right => U8Code::TriU8(RIGHT),
            KeyCode::Up => U8Code::TriU8(UP),
            KeyCode::Down => U8Code::TriU8(DOWN),
            KeyCode::Home => U8Code::TriU8(HOME),
            KeyCode::End => U8Code::TriU8(END),
            KeyCode::PageUp => todo!(),
            KeyCode::PageDown => todo!(),
            KeyCode::Tab => U8Code::Ascii(ascii::TAB),
            KeyCode::BackTab => U8Code::TriU8(BACKTAB),
            KeyCode::Delete => U8Code::Ascii(ascii::DELETE),
            KeyCode::Insert => todo!(),
            KeyCode::F(num) => U8Code::TriU8(function::f(num)),
            KeyCode::Null => U8Code::Ascii(ascii::NULL),
            KeyCode::Esc => U8Code::Ascii(ascii::ESC),

            KeyCode::CapsLock
            | KeyCode::ScrollLock
            | KeyCode::NumLock
            | KeyCode::PrintScreen
            | KeyCode::Pause
            | KeyCode::Menu
            | KeyCode::KeypadBegin
            | KeyCode::Media(_)
            | KeyCode::Modifier(_) => {
                unreachable!("KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES is not enabled")
            }
        }
    }
}
pub mod ascii {
    pub fn ctrl(ch: char) -> Option<u8> {
        if ch.is_ascii_digit() {
            Some(ch as u8)
        } else if ch.is_ascii_lowercase() {
            Some(ch as u8 - b'a' + 1)
        } else if ch.is_ascii_uppercase() {
            Some(ch as u8 - b'A' + 1)
        } else if ch == '+' {
            Some(ch as u8)
        } else if ch == '-' {
            Some(31)
        } else {
            None
        }
    }
    pub fn alt(ch: char) -> Option<[u8; 3]> {
        if ch.is_ascii_digit() {
            Some([ctrl(ch)?, 0, 0])
        } else if ch.is_ascii_lowercase() | ch.is_ascii_uppercase() {
            Some([27, ch as u8, 0])
        } else {
            None
        }
    }
    pub const NULL: u8 = 0;
    pub const SPACE: u8 = 32;
    pub const BACKSPACE: u8 = 8;
    pub const TAB: u8 = 9;
    pub const ENTER: u8 = 10;
    pub const ESC: u8 = 27;
    pub const DELETE: u8 = 127;
}
pub mod function {
    const F1: [u8; 3] = [27, 79, 80];
    const F5: [u8; 3] = [53, 126, 0];
    const F9: [u8; 3] = [48, 126, 0];
    pub fn f(num: u8) -> [u8; 3] {
        match num {
            1_u8..=4_u8 => {
                let mut raw = F1;
                raw[2] += num - 1;
                raw
            }
            5_u8..=8_u8 => {
                let mut raw = F5;
                raw[0] += num - 1;
                raw
            }
            9_u8..12_u8 => {
                let mut raw = F9;
                raw[0] += num - 1;
                raw
            }
            _ => unimplemented!("Only F1-12"),
        }
    }
}
pub const UP: [u8; 3] = [27, 91, 65];
pub const DOWN: [u8; 3] = [27, 91, 66];
pub const RIGHT: [u8; 3] = [27, 91, 67];
pub const LEFT: [u8; 3] = [27, 91, 68];
pub const BACKTAB: [u8; 3] = [27, 91, 90];
pub const END: [u8; 3] = [27, 91, 70];
pub const HOME: [u8; 3] = [27, 91, 72];
