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

fn character_to_pitch(character: u8) -> Option<isize> {
    match character {
        b'c' => Some(3),
        b'd' => Some(5),
        b'e' => Some(7),
        b'f' => Some(8),
        b'g' => Some(10),
        b'a' => Some(12),
        b'b' => Some(14),
        _ => None,
    }
}

pub fn chord(stream: &mut RollbackableTokenStream) -> ParseResult {
    if !stream.expect_character(b'(') {
        return None;
    }

    let mut notes = Vec::new();
    let mut octave = 0;

    loop {
        match stream.take_character() {
            Some(b')') => break,
            Some(b'<') => octave += 1,
            Some(b'>') => octave -= 1,
            None => return Some(Err("Unexpected EOF after (".to_string())),
            Some(x) => {
                if let Some(pitch) = character_to_pitch(x) {
                    notes.push(pitch + octave * 12);
                } else {
                    return Some(Err(format!("Unexpected token {}", x)))
                }
            }
        }
    }

    let length = parse_length(stream);

    Some(Ok(Instruction::Chord(notes, length)))
}

pub fn note(stream: &mut RollbackableTokenStream) -> ParseResult {
    let character = stream.take_character()?;
    let mut pitch = character_to_pitch(character)?;

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
