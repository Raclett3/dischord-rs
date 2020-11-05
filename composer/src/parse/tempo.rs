use crate::parse::{Instruction, ParseResult, RollbackableTokenStream};
use crate::tokenize::TokenKind;

pub fn tempo(stream: &mut RollbackableTokenStream) -> ParseResult {
    let (_, token) = stream.next()?;
    if let TokenKind::Character(ch) = token {
        if ch == b't' {
            let tempo = stream.next();
            match tempo {
                Some((_, TokenKind::Number(num))) => Some(Ok(Instruction::Tempo(num))),
                Some((token_at, token)) => {
                    Some(Err(format!("Unexpected token {} at {}", token_at, token)))
                }
                _ => Some(Err(format!("Unexpected EOF after the token {}", token))),
            }
        } else {
            None
        }
    } else {
        None
    }
}
