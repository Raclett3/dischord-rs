use crate::parse::{Instruction, ParseResult, RollbackableTokenStream};
use crate::tokenize::TokenKind;

pub fn tone(stream: &mut RollbackableTokenStream) -> ParseResult {
    if !stream.expect_character(b'@') {
        return None;
    }

    match stream.next() {
        Some((_, TokenKind::Number(num))) => Some(Ok(Instruction::Tone(num))),
        Some((token_at, TokenKind::Character(ch))) => {
            Some(Err(format!("Unexpected token {} at {}", ch, token_at)))
        }
        _ => Some(Err("Unexpected EOF after the token @".to_string())),
    }
}
