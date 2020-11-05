use crate::parse::{Instruction, ParseResult, RollbackableTokenStream};
use crate::tokenize::TokenKind;

pub fn octave(stream: &mut RollbackableTokenStream) -> ParseResult {
    let (_, token) = stream.next()?;
    match token {
        TokenKind::Character(b'<') => Some(Ok(Instruction::Octave(1))),
        TokenKind::Character(b'>') => Some(Ok(Instruction::Octave(-1))),
        _ => None,
    }
}
