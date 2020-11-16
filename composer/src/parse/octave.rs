use crate::parse::{Instruction, ParseResult, RollbackableTokenStream};

pub fn octave(stream: &mut RollbackableTokenStream) -> ParseResult {
    match stream.take_character() {
        Ok((_, '<')) => Ok(Some(Instruction::Octave(1))),
        Ok((_, '>')) => Ok(Some(Instruction::Octave(-1))),
        _ => Ok(None),
    }
}
