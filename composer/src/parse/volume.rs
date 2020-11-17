use crate::parse::{Instruction, ParseResult, RollbackableTokenStream};

pub fn volume(stream: &mut RollbackableTokenStream) -> ParseResult {
    if stream.expect_character('v').is_err() {
        return Ok(None);
    }

    let (_, volume) = stream.take_number()?;
    Ok(Some(Instruction::Volume(volume as f32 / 100.0)))
}
