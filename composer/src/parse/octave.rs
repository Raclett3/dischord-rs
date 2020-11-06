use crate::parse::{Instruction, ParseResult, RollbackableTokenStream};

pub fn octave(stream: &mut RollbackableTokenStream) -> ParseResult {
    match stream.take_character() {
        Some(b'<') => Some(Ok(Instruction::Octave(1))),
        Some(b'>') => Some(Ok(Instruction::Octave(-1))),
        _ => None,
    }
}
