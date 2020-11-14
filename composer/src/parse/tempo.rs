use crate::parse::{Instruction, ParseResult, RollbackableTokenStream};
use crate::tokenize::TokenKind;

pub fn tempo(stream: &mut RollbackableTokenStream) -> ParseResult {
    if !stream.expect_character('t') {
        return None;
    }

    let tempo = stream.next();
    match tempo {
        Some(&(_, TokenKind::Number(num))) => Some(Ok(Instruction::Tempo(num))),
        Some((token_at, token)) => Some(Err(format!("Unexpected token {} at {}", token_at, token))),
        _ => Some(Err("Unexpected EOF after the token T".to_string())),
    }
}
