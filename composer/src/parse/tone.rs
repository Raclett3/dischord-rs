use crate::parse::{Instruction, ParseResult, RollbackableTokenStream};
use crate::tokenize::TokenKind;

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
            Some(Ok(Instruction::Detune(params[0], params[1] as f64 / 10000.0)))
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
        Some((token_at, token)) => {
            Some(Err(format!("Unexpected token {} at {}", token, token_at)))
        }
        None => Some(Err("Unexpected EOF after the token @".to_string())),
    }
}
