use crate::parse::{Instruction, ParseResult, RollbackableTokenStream};

pub fn tempo(stream: &mut RollbackableTokenStream) -> ParseResult {
    if stream.expect_character('t').is_err() {
        return Ok(None);
    }

    let (_, tempo) = stream.take_number()?;
    Ok(Some(Instruction::Tempo(tempo)))
}
