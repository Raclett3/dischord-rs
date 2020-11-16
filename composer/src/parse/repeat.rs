use crate::parse::{parse_stream, Instruction, ParseResult, RollbackableTokenStream};

pub fn repeat(stream: &mut RollbackableTokenStream) -> ParseResult {
    if stream.expect_character('[').is_err() {
        return Ok(None);
    }

    let mut cloned_stream = stream.clone();
    cloned_stream.accept();

    let inside = parse_stream(&mut cloned_stream, true)?.remove(0); // Take the first track

    let (_, num) = cloned_stream.take_number()?;
    *stream = cloned_stream;
    Ok(Some(Instruction::Repeat(inside, num)))
}
