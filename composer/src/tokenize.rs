#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    Character(u8),
    Number(usize),
}

pub fn tokenize(mml: &str) -> Result<Vec<Token>, String> {
    let is_ascii = mml.chars().all(|x| x <= '\u{7f}');
    if !is_ascii {
        return Err("MML must not include any non-ascii characters".to_string());
    }

    let mut bytes = mml.bytes().enumerate().peekable();
    let mut tokens = Vec::new();

    while let Some((i, byte)) = bytes.next() {
        if b'0' <= byte && byte <= b'9' {
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

            tokens.push(Token::Number(number));
        } else if byte != b' ' {
            tokens.push(Token::Character(byte));
        }
    }
    Ok(tokens)
}
