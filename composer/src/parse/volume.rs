use crate::parse::{Instruction, ParseResult, RollbackableTokenStream};
use crate::tokenize::TokenKind;

pub fn volume(stream: &mut RollbackableTokenStream) -> ParseResult {
    if !stream.expect_character(b'v') {
        return None;
    }

    let tempo = stream.next();
    match tempo {
        Some((_, TokenKind::Number(num))) => Some(Ok(Instruction::Volume(num))),
        Some((token_at, token)) => Some(Err(format!("Unexpected token {} at {}", token_at, token))),
        _ => Some(Err("Unexpected EOF after the token V".to_string())),
    }
}
