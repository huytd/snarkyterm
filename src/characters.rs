use winit::event::{ModifiersState, VirtualKeyCode::{self, *}};

pub const NEWLINE_CHAR: char = 10 as char;
pub const SPACE_CHAR: char = 32 as char;
pub const TAB_CHAR: char = 9 as char;
pub const BACK_CHAR: char = 8 as char;
pub const ESC_CHAR: char = 27 as char;
pub const CR_CHAR: char = 13 as char;
pub const BELL_CHAR: char = 7 as char;

pub struct InputChar {}
impl InputChar {
    pub fn from(key: VirtualKeyCode, modifiers: ModifiersState) -> Option<char> {
        match key {
            Key0 if modifiers.shift() => Some(')'),
            Key1 if modifiers.shift() => Some('!'),
            Key2 if modifiers.shift() => Some('@'),
            Key3 if modifiers.shift() => Some('#'),
            Key4 if modifiers.shift() => Some('$'),
            Key5 if modifiers.shift() => Some('%'),
            Key6 if modifiers.shift() => Some('^'),
            Key7 if modifiers.shift() => Some('&'),
            Key8 if modifiers.shift() => Some('*'),
            Key9 if modifiers.shift() => Some('('),

            Key0 | Numpad0 => Some('0'),
            Key1 | Numpad1 => Some('1'),
            Key2 | Numpad2 => Some('2'),
            Key3 | Numpad3 => Some('3'),
            Key4 | Numpad4 => Some('4'),
            Key5 | Numpad5 => Some('5'),
            Key6 | Numpad6 => Some('6'),
            Key7 | Numpad7 => Some('7'),
            Key8 | Numpad8 => Some('8'),
            Key9 | Numpad9 => Some('9'),

            A if modifiers.shift() => Some('A'),
            B if modifiers.shift() => Some('B'),
            C if modifiers.shift() => Some('C'),
            D if modifiers.shift() => Some('D'),
            E if modifiers.shift() => Some('E'),
            F if modifiers.shift() => Some('F'),
            G if modifiers.shift() => Some('G'),
            H if modifiers.shift() => Some('H'),
            I if modifiers.shift() => Some('I'),
            J if modifiers.shift() => Some('J'),
            K if modifiers.shift() => Some('K'),
            L if modifiers.shift() => Some('L'),
            M if modifiers.shift() => Some('M'),
            N if modifiers.shift() => Some('N'),
            O if modifiers.shift() => Some('O'),
            P if modifiers.shift() => Some('P'),
            Q if modifiers.shift() => Some('Q'),
            R if modifiers.shift() => Some('R'),
            S if modifiers.shift() => Some('S'),
            T if modifiers.shift() => Some('T'),
            U if modifiers.shift() => Some('U'),
            V if modifiers.shift() => Some('V'),
            W if modifiers.shift() => Some('W'),
            X if modifiers.shift() => Some('X'),
            Y if modifiers.shift() => Some('Y'),
            Z if modifiers.shift() => Some('Z'),

            A if modifiers.ctrl() => Some(0x01 as char),
            B if modifiers.ctrl() => Some(0x02 as char),
            C if modifiers.ctrl() => Some(0x03 as char),
            D if modifiers.ctrl() => Some(0x04 as char),
            E if modifiers.ctrl() => Some(0x05 as char),
            F if modifiers.ctrl() => Some(0x06 as char),
            G if modifiers.ctrl() => Some(0x07 as char),
            H if modifiers.ctrl() => Some(0x08 as char),
            I if modifiers.ctrl() => Some(0x09 as char),
            J if modifiers.ctrl() => Some(0x10 as char),
            K if modifiers.ctrl() => Some(0x11 as char),
            L if modifiers.ctrl() => Some(0x12 as char),
            M if modifiers.ctrl() => Some(0x13 as char),
            N if modifiers.ctrl() => Some(0x14 as char),
            O if modifiers.ctrl() => Some(0x15 as char),
            P if modifiers.ctrl() => Some(0x16 as char),
            Q if modifiers.ctrl() => Some(0x17 as char),
            R if modifiers.ctrl() => Some(0x18 as char),
            S if modifiers.ctrl() => Some(0x19 as char),
            T if modifiers.ctrl() => Some(0x1a as char),
            U if modifiers.ctrl() => Some(0x21 as char),
            V if modifiers.ctrl() => Some(0x22 as char),
            W if modifiers.ctrl() => Some(0x23 as char),
            X if modifiers.ctrl() => Some(0x24 as char),
            Y if modifiers.ctrl() => Some(0x25 as char),
            Z if modifiers.ctrl() => Some(0x26 as char),

            A => Some('a'),
            B => Some('b'),
            C => Some('c'),
            D => Some('d'),
            E => Some('e'),
            F => Some('f'),
            G => Some('g'),
            H => Some('h'),
            I => Some('i'),
            J => Some('j'),
            K => Some('k'),
            L => Some('l'),
            M => Some('m'),
            N => Some('n'),
            O => Some('o'),
            P => Some('p'),
            Q => Some('q'),
            R => Some('r'),
            S => Some('s'),
            T => Some('t'),
            U => Some('u'),
            V => Some('v'),
            W => Some('w'),
            X => Some('x'),
            Y => Some('y'),
            Z => Some('z'),

            Tab => Some(TAB_CHAR),
            Back => Some(BACK_CHAR),
            Space => Some(' '),

            Minus if modifiers.shift() => Some('_'),
            Minus => Some('-'),
            Plus => Some('+'),
            Equals if modifiers.shift() => Some('+'),
            Equals => Some('='),
            Grave if modifiers.shift() => Some('~'),
            Grave => Some('`'),
            Slash if modifiers.shift() => Some('?'),
            Slash => Some('/'),
            Backslash if modifiers.shift() => Some('|'),
            Backslash => Some('\\'),
            Colon => Some(':'),
            Semicolon if modifiers.shift() => Some(':'),
            Semicolon => Some(';'),
            Asterisk => Some('*'),
            Apostrophe if modifiers.shift() => Some('"'),
            Apostrophe => Some('\''),
            Comma if modifiers.shift() => Some('<'),
            Comma => Some(','),
            Period if modifiers.shift() => Some('>'),
            Period => Some('.'),
            LBracket if modifiers.shift() => Some('{'),
            LBracket => Some('['),
            RBracket if modifiers.shift() => Some('}'),
            RBracket => Some(']'),

            Return => Some('\n'),

            _ => None
        }
    }
}

pub struct EscapeCode {}
impl EscapeCode {
    fn parse_param(input: &[u8]) -> (&[u8], Vec<u8>) {
        if input.len() > 0 {
            let found: Vec<u8> = input.iter().by_ref().take_while(|&c| *c >= 0x30 && *c <= 0x3F).cloned().collect();
            let remain = &input[found.len()..];
            return (remain, found);
        }
        (input, vec![])
    }

    fn parse_intermediates(input: &[u8]) -> (&[u8], Vec<u8>) {
        if input.len() > 0 {
            let found: Vec<u8> = input.iter().by_ref().take_while(|&c| *c >= 0x20 && *c <= 0x2F).cloned().collect();
            let remain = &input[found.len()..];
            return (remain, found);
        }
        (input, vec![])
    }

    fn parse_final(input: &[u8]) -> (&[u8], u8) {
        if input.len() > 0 {
            let found: Vec<u8> = input.iter().by_ref().take_while(|&c| *c >= 0x40 && *c <= 0x7E).cloned().collect();
            let remain = &input[1..];
            return (remain, found[0]);
        }
        (input, 0)
    }

    pub fn parse_csi(buf: &[u8]) -> (&[u8], Vec<u8>, Vec<u8>, u8) {
        if buf.len() > 0 && buf[0] == '[' as u8 {
            let (remain, param) = Self::parse_param(&buf[1..]);
            let (remain, intermediates) = Self::parse_intermediates(remain);
            let (remain, final_byte) = Self::parse_final(remain);
            return (remain, param, intermediates, final_byte);
        }
        (buf, vec![], vec![], 0)
    }
}
