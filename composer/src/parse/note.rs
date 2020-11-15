use crate::parse::{Instruction, NoteLength, ParseResult, RollbackableTokenStream};
use crate::tokenize::TokenKind;

pub fn parse_length(stream: &mut RollbackableTokenStream) -> Vec<NoteLength> {
    let mut length = vec![];
    loop {
        if let Some(&(_, TokenKind::Number(num))) = stream.peek() {
            stream.next();
            length.push(NoteLength::Length(num))
        } else {
            length.push(NoteLength::DefaultLength);
        }

        while let Some((_, TokenKind::Character('.'))) = stream.peek() {
            stream.next();
            length.push(NoteLength::Dot)
        }

        if let Some((_, TokenKind::Character('&'))) = stream.peek() {
            stream.next();
        } else {
            break;
        }
    }
    length
}

pub fn rest(stream: &mut RollbackableTokenStream) -> ParseResult {
    if !stream.expect_character('r') {
        return None;
    }

    let length = parse_length(stream);

    Some(Ok(Instruction::Rest(length)))
}

pub fn length(stream: &mut RollbackableTokenStream) -> ParseResult {
    if !stream.expect_character('l') {
        return None;
    }

    let length = parse_length(stream);

    Some(Ok(Instruction::Length(length)))
}

fn character_to_pitch(character: char) -> Option<isize> {
    match character {
        'c' => Some(3),
        'd' => Some(5),
        'e' => Some(7),
        'f' => Some(8),
        'g' => Some(10),
        'a' => Some(12),
        'b' => Some(14),
        _ => None,
    }
}

pub fn chord(stream: &mut RollbackableTokenStream) -> ParseResult {
    if !stream.expect_character('(') {
        return None;
    }

    let mut notes = Vec::new();
    let mut octave = 0;

    loop {
        match stream.take_character() {
            Some((_, ')')) => break,
            Some((_, '<')) => octave += 1,
            Some((_, '>')) => octave -= 1,
            None => return Some(Err("Unexpected EOF after (".to_string())),
            Some((token_at, x)) => {
                if let Some(mut pitch) = character_to_pitch(x) {
                    loop {
                        if stream.expect_character('+') {
                            pitch += 1;
                        } else if stream.expect_character('-') {
                            pitch -= 1;
                        } else {
                            break;
                        }
                    }
                    notes.push(pitch + octave * 12);
                } else {
                    return Some(Err(format!("Unexpected token {} at {}", x, token_at)));
                }
            }
        }
    }

    let length = parse_length(stream);

    Some(Ok(Instruction::Chord(notes, length)))
}

pub fn note(stream: &mut RollbackableTokenStream) -> ParseResult {
    let (_, character) = stream.take_character()?;
    let mut pitch = character_to_pitch(character)?;

    loop {
        match stream.peek() {
            Some((_, TokenKind::Character('+'))) => pitch += 1,
            Some((_, TokenKind::Character('-'))) => pitch -= 1,
            _ => break,
        }

        stream.next();
    }

    let length = parse_length(stream);

    Some(Ok(Instruction::Note(pitch, length)))
}
