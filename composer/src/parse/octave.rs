use crate::parse::{Instruction, ParseResult, RollbackableTokenStream};

pub fn octave(stream: &mut RollbackableTokenStream) -> ParseResult {
    match stream.take_character() {
        Some((_, '<')) => Some(Ok(Instruction::Octave(1))),
        Some((_, '>')) => Some(Ok(Instruction::Octave(-1))),
        _ => None,
    }
}
