#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Character(char),
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

    let mut chars = mml.chars().enumerate().peekable();
    let mut tokens = Vec::new();

    while let Some((i, byte)) = chars.next() {
        let token = if '0' <= byte && byte <= '9' {
            let mut number = (byte as u8 - b'0') as usize;

            while let Some(&(_, peeked)) = chars.peek() {
                if !('0' <= peeked && peeked <= '9') {
                    break;
                }
                let (multiplied, mul_overflowed) = number.overflowing_mul(10);
                let (added, add_overflowed) =
                    multiplied.overflowing_add((peeked as u8 - b'0') as usize);

                if mul_overflowed || add_overflowed {
                    return Err(format!("Too big number at {}", i + 1));
                }

                number = added;

                chars.next();
            }

            TokenKind::Number(number)
        } else if 'A' <= byte && byte <= 'Z' {
            TokenKind::Character(byte.to_lowercase().next().unwrap())
        } else if byte == ' ' || byte == '\n' || byte == '\r' {
            continue;
        } else {
            TokenKind::Character(byte)
        };

        tokens.push((i + 1, token));
    }
    Ok(tokens)
}
