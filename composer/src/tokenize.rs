#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Character(u8),
    Number(usize),
}

impl TokenKind {
    pub fn is_character(&self) -> bool {
        match self {
            TokenKind::Character(_) => true,
            TokenKind::Number(_) => false,
        }
    }

    pub fn is_number(&self) -> bool {
        match self {
            TokenKind::Number(_) => true,
            TokenKind::Character(_) => false,
        }
    }
}

use std::fmt;

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Character(x) => write!(f, "{}", x),
            TokenKind::Number(x) => write!(f, "{}", x),
        }
    }
}

pub type Token = (usize, TokenKind);

pub fn tokenize(mml: &str) -> Result<Vec<Token>, String> {
    let is_ascii = mml.chars().all(|x| x <= '\u{7f}');
    if !is_ascii {
        return Err("MML must not include any non-ascii characters".to_string());
    }

    let mut bytes = mml.bytes().enumerate().peekable();
    let mut tokens = Vec::new();

    while let Some((i, byte)) = bytes.next() {
        let token = if b'0' <= byte && byte <= b'9' {
            let mut number = (byte - b'0') as usize;

            while let Some(&(_, peeked)) = bytes.peek() {
                if !(b'0' <= peeked && peeked <= b'9') {
                    break;
                }
                let (multiplied, mul_overflowed) = number.overflowing_mul(10);
                let (added, add_overflowed) = multiplied.overflowing_add((peeked - b'0') as usize);

                if mul_overflowed || add_overflowed {
                    return Err(format!("Too big number at {}", i + 1));
                }

                number = added;

                bytes.next();
            }

            TokenKind::Number(number)
        } else if b'A' <= byte && byte <= b'Z' {
            TokenKind::Character(byte - b'A' + b'a')
        } else if byte == b' ' || byte == b'\n' || byte == b'\r' {
            continue;
        } else {
            TokenKind::Character(byte)
        };

        tokens.push((i + 1, token));
    }
    Ok(tokens)
}
