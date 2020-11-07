use crate::parse::{parse_stream, Instruction, ParseResult, RollbackableTokenStream};
use crate::tokenize::TokenKind;

pub fn repeat(stream: &mut RollbackableTokenStream) -> ParseResult {
    if !stream.expect_character('[') {
        return None;
    }

    let mut cloned_stream = stream.clone();
    cloned_stream.accept();

    let inside = match parse_stream(&mut cloned_stream, true) {
        Ok(mut parsed) => parsed.remove(0),
        Err(err) => return Some(Err(err)),
    };

    match cloned_stream.next() {
        Some((_, TokenKind::Number(num))) => {
            *stream = cloned_stream;
            Some(Ok(Instruction::Repeat(inside, num)))
        },
        Some((token_at, token)) => Some(Err(format!("Unexpected token {} at {}", token, token_at))),
        None => Some(Err("Unexpected EOF".to_string()))
    }
}
