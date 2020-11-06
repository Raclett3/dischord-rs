use crate::parse::{Instruction, NoteLength, ParseResult, RollbackableTokenStream};
use crate::tokenize::TokenKind;

pub fn parse_length(stream: &mut RollbackableTokenStream) -> Vec<NoteLength> {
    let mut length = vec![];
    loop {
        if let Some((_, TokenKind::Number(num))) = stream.peek() {
            stream.next();
            length.push(NoteLength::Length(num))
        } else {
            length.push(NoteLength::DefaultLength);
        }

        while let Some((_, TokenKind::Character(b'.'))) = stream.peek() {
            stream.next();
            length.push(NoteLength::Dot)
        }

        if let Some((_, TokenKind::Character(b'&'))) = stream.peek() {
            stream.next();
        } else {
            break;
        }
    }
    length
}

pub fn rest(stream: &mut RollbackableTokenStream) -> ParseResult {
    if !stream.expect_character(b'r') {
        return None;
    }

    let length = parse_length(stream);

    Some(Ok(Instruction::Rest(length)))
}

pub fn length(stream: &mut RollbackableTokenStream) -> ParseResult {
    if !stream.expect_character(b'l') {
        return None;
    }

    let length = parse_length(stream);

    Some(Ok(Instruction::Rest(length)))
}

pub fn note(stream: &mut RollbackableTokenStream) -> ParseResult {
    let character = stream.take_character()?;
    let mut pitch = match character {
        b'c' => 3,
        b'd' => 5,
        b'e' => 7,
        b'f' => 8,
        b'g' => 10,
        b'a' => 12,
        b'b' => 14,
        _ => return None,
    };

    loop {
        match stream.peek() {
            Some((_, TokenKind::Character(b'+'))) => pitch += 1,
            Some((_, TokenKind::Character(b'-'))) => pitch -= 1,
            _ => break,
        }

        stream.next();
    }

    let length = parse_length(stream);

    Some(Ok(Instruction::Note(pitch, length)))
}
