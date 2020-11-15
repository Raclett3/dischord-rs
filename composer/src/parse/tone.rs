use crate::parse::{Instruction, ParseResult, RollbackableTokenStream};
use crate::tokenize::TokenKind;

fn hex_to_num(hex: u8) -> Option<usize> {
    if b'0' <= hex && hex <= b'9' {
        Some((hex - b'0') as usize)
    } else if b'a' <= hex && hex <= b'f' {
        Some((hex - b'a') as usize + 10)
    } else {
        None
    }
}

pub fn tone(stream: &mut RollbackableTokenStream) -> ParseResult {
    if !stream.expect_character('@') {
        return None;
    }

    match stream.next() {
        Some(&(_, TokenKind::Number(num))) => Some(Ok(Instruction::Tone(num))),
        Some((token_at, TokenKind::Character('d'))) => {
            let params = stream.comma_separated_numbers();
            if params.len() != 2 {
                let err = format!("Wrong number of parameters at {}", token_at);
                return Some(Err(err));
            }
            Some(Ok(Instruction::Detune(
                params[0],
                params[1] as f64 / 10000.0,
            )))
        }
        Some((token_at, TokenKind::Character('e'))) => {
            let params = stream.comma_separated_numbers();
            if params.len() != 4 {
                let err = format!("Wrong number of parameters at {}", token_at);
                return Some(Err(err));
            }
            let params: Vec<_> = params.iter().map(|&x| x as f64 / 100.0).collect();
            let envelope = Instruction::Envelope(params[0], params[1], params[2], params[3]);
            Some(Ok(envelope))
        }
        Some((_, TokenKind::Character('h'))) => {
            let pcm = match stream.next() {
                Some((_, TokenKind::BraceString(string))) => string
                    .bytes()
                    .map(|byte| {
                        hex_to_num(byte)
                            .map(|x| (x as f64 - 8.0) / 8.0)
                            .unwrap_or(0.0)
                    })
                    .collect(),
                Some((token_at, token)) => {
                    return Some(Err(format!("Unexpected token {} at {}", token, token_at)));
                }
                None => return Some(Err("Unexpected EOF after the token @h".to_string())),
            };
            Some(Ok(Instruction::DefinePCMTone(pcm)))
        }
        Some((_, TokenKind::Character('p'))) => {
            let num = match stream.next() {
                Some((_, TokenKind::Number(num))) => *num,
                Some((token_at, token)) => {
                    return Some(Err(format!("Unexpected token {} at {}", token, token_at)));
                }
                None => return Some(Err("Unexpected EOF after the token @p".to_string())),
            };
            Some(Ok(Instruction::PCMTone(num)))
        }
        Some((_, TokenKind::Character('g'))) => {
            let gate = match stream.next() {
                Some((_, TokenKind::Number(x))) => *x,
                Some((token_at, token)) => {
                    return Some(Err(format!("Unexpected token {} at {}", token, token_at)));
                }
                None => return Some(Err("Unexpected EOF after the token @p".to_string())),
            };
            Some(Ok(Instruction::Gate(gate as f64 / 1000.0)))
        }
        Some((token_at, token)) => Some(Err(format!("Unexpected token {} at {}", token, token_at))),
        None => Some(Err("Unexpected EOF after the token @".to_string())),
    }
}
