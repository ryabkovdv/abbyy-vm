use std::error;
use std::fmt;
use std::marker::PhantomData;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    UnknownToken,
    UnterminatedString,
    InvalidCharLiteral,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Token<'a> {
    Eol,
    Eof,
    Err(Error),

    Char(char),
    Reg(&'a str),
    Int(&'a str),
    Str(&'a str),
    Ident(&'a str),
    Label(&'a str),

    Comma,
    Equal,
    Lparen,
    Rparen,

    Or,
    Xor,

    Add,
    Sub,

    And,
    Mul,

    Shl,
    Lshr,
    Ashr,
}

pub struct Lexer<'a> {
    cur: *const u8,
    end: *const u8,
    token: Token<'a>,
    _data: PhantomData<&'a [u8]>,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Lexer<'a> {
        unsafe {
            let (start, len) = (src.as_ptr(), src.len());
            let end = start.add(len);

            let (token, cur) = scan_token(start, end);
            Lexer { cur, end, token, _data: PhantomData }
        }
    }

    pub fn peek(&self) -> Token<'a> {
        self.token
    }

    pub fn next(&mut self) -> Token<'a> {
        let old_token = self.token;
        unsafe {
            let (token, cur) = scan_token(self.cur, self.end);
            self.cur = cur;
            self.token = token;
        }
        old_token
    }
}

unsafe fn scan_token<'a>(mut start: *const u8, end: *const u8) -> (Token<'a>, *const u8) {
    while start < end {
        if !matches!(*start, b' ' | b'\t' | b'\r') {
            break;
        }
        start = start.add(1);
    }

    if start >= end {
        return (Token::Eof, start);
    }

    unsafe fn make_str<'a>(start: *const u8, end: *const u8) -> &'a str {
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(
            start,
            end.offset_from(start) as usize,
        ))
    }

    let start = start;
    let mut cur = start;

    let byte = *cur;
    cur = cur.add(1);
    match byte {
        b'0'..=b'9' => {
            while cur < end {
                if !is_ident(*cur) {
                    break;
                }
                cur = cur.add(1);
            }
            (Token::Int(make_str(start, cur)), cur)
        }
        b'A'..=b'Z' | b'a'..=b'z' | b'.' | b'_' => {
            while cur < end {
                if !is_ident(*cur) {
                    break;
                }
                cur = cur.add(1);
            }

            let s = make_str(start, cur);
            if cur < end && *cur == b':' {
                (Token::Label(s), cur.add(1))
            } else {
                (Token::Ident(s), cur)
            }
        }
        b'%' => {
            while cur < end {
                if !is_ident(*cur) {
                    break;
                }
                cur = cur.add(1);
            }
            (Token::Reg(make_str(start.add(1), cur)), cur)
        }
        b'"' => {
            while cur < end {
                match *cur {
                    b'"' => return (Token::Str(make_str(start.add(1), cur)), cur.add(1)),
                    b'\n' => break,
                    _ => {}
                }
                cur = cur.add(1);
            }
            (Token::Err(Error::UnterminatedString), cur)
        }
        b'\'' => {
            if cur < end {
                let (ch, new_cur) = decode_char(cur);
                cur = new_cur;

                if cur < end && *cur == b'\'' {
                    if ch <= '\u{001F}' {
                        return (Token::Err(Error::InvalidCharLiteral), cur.add(1));
                    }
                    return (Token::Char(ch), cur.add(1));
                }
            }
            (Token::Err(Error::InvalidCharLiteral), cur)
        }
        b';' => {
            while cur < end {
                if *cur == b'\n' {
                    cur = cur.add(1);
                    break;
                }
                cur = cur.add(1);
            }
            (Token::Eol, cur)
        }
        b'<' => {
            if cur < end && *cur == b'<' {
                return (Token::Shl, cur.add(1));
            }
            (Token::Err(Error::UnknownToken), cur)
        }
        b'>' => {
            if cur < end && *cur == b'>' {
                cur = cur.add(1);
                if cur < end && *cur == b'>' {
                    return (Token::Lshr, cur.add(1));
                }
                return (Token::Ashr, cur);
            }
            (Token::Err(Error::UnknownToken), cur)
        }
        b'\n' => (Token::Eol, cur),
        b',' => (Token::Comma, cur),
        b'=' => (Token::Equal, cur),
        b'(' => (Token::Lparen, cur),
        b')' => (Token::Rparen, cur),
        b'|' => (Token::Or, cur),
        b'^' => (Token::Xor, cur),
        b'+' => (Token::Add, cur),
        b'-' => (Token::Sub, cur),
        b'&' => (Token::And, cur),
        b'*' => (Token::Mul, cur),
        _ => (Token::Err(Error::UnknownToken), cur),
    }
}

unsafe fn decode_char(ptr: *const u8) -> (char, *const u8) {
    let byte = *ptr;
    match byte {
        0..=0x7F => (byte as char, ptr.add(1)),
        0..=0b1101_1111 => {
            debug_assert!(byte >= 0b1100_0000);
            let p1 = (byte & 0b0001_1111) as u32;
            let p2 = (*ptr.add(1) & 0b0011_1111) as u32;
            let ch = (p1 << 6) | p2;
            (char::from_u32_unchecked(ch), ptr.add(2))
        }
        0..=0b1110_1111 => {
            let p1 = (byte & 0b0000_1111) as u32;
            let p2 = (*ptr.add(1) & 0b0011_1111) as u32;
            let p3 = (*ptr.add(2) & 0b0011_1111) as u32;
            let ch = (p1 << 12) | (p2 << 6) | p3;
            (char::from_u32_unchecked(ch), ptr.add(3))
        }
        _ => {
            debug_assert!(byte <= 0b1111_0111);
            let p1 = (byte & 0b0000_0111) as u32;
            let p2 = (*ptr.add(1) & 0b0011_1111) as u32;
            let p3 = (*ptr.add(2) & 0b0011_1111) as u32;
            let p4 = (*ptr.add(3) & 0b0011_1111) as u32;
            let ch = (p1 << 18) | (p2 << 12) | (p3 << 6) | p4;
            (char::from_u32_unchecked(ch), ptr.add(4))
        }
    }
}

fn is_ident(byte: u8) -> bool {
    const LUT: [usize; (256 / usize::BITS) as usize] = {
        let mut table = [0_usize; (256 / usize::BITS) as usize];

        let mut idx: usize = 0;
        while idx < table.len() {
            let mut bit: u32 = 0;
            while bit < usize::BITS {
                match ((idx as u32) * usize::BITS + bit) as u8 {
                    b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'.' | b'_' => {
                        table[idx] |= 1 << bit;
                    }
                    _ => {}
                }
                bit += 1;
            }
            idx += 1;
        }

        table
    };

    let idx = byte / (usize::BITS as u8);
    let bit = byte % (usize::BITS as u8);
    (LUT[idx as usize] & (1 << bit)) != 0
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;
        let msg = match self {
            UnknownToken       => "unknown token",
            UnterminatedString => "unterminated string",
            InvalidCharLiteral => "invalid character literal",
        };
        f.write_str(msg)
    }
}

impl error::Error for Error {}
