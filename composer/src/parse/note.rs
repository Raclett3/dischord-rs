use crate::parse::{Instruction, NoteLength, ParseError, ParseResult, RollbackableTokenStream};
use crate::tokenize::TokenKind;
use crate::try_or_ok_none;

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
    if stream.expect_character('r').is_err() {
        return Ok(None);
    }

    let length = parse_length(stream);

    Ok(Some(Instruction::Rest(length)))
}

pub fn length(stream: &mut RollbackableTokenStream) -> ParseResult {
    if stream.expect_character('l').is_err() {
        return Ok(None);
    }

    let length = parse_length(stream);

    Ok(Some(Instruction::Length(length)))
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
    if stream.expect_character('(').is_err() {
        return Ok(None);
    }

    let mut notes = Vec::new();
    let mut octave = 0;

    loop {
        match stream.take_character()? {
            (_, ')') => break,
            (_, '<') => octave += 1,
            (_, '>') => octave -= 1,
            (token_at, x) => {
                if let Some(mut pitch) = character_to_pitch(x) {
                    loop {
                        if stream.expect_character('+').is_ok() {
                            pitch += 1;
                        } else if stream.expect_character('-').is_ok() {
                            pitch -= 1;
                        } else {
                            break;
                        }
                    }
                    notes.push(pitch + octave * 12);
                } else {
                    return Err(ParseError::unexpected_char(token_at, x));
                }
            }
        }
    }

    let length = parse_length(stream);

    Ok(Some(Instruction::Chord(notes, length)))
}

pub fn note(stream: &mut RollbackableTokenStream) -> ParseResult {
    let (_, character) = try_or_ok_none!(stream.take_character());
    let mut pitch = if let Some(pitch) = character_to_pitch(character) {
        pitch
    } else {
        return Ok(None);
    };

    loop {
        match stream.peek() {
            Some((_, TokenKind::Character('+'))) => pitch += 1,
            Some((_, TokenKind::Character('-'))) => pitch -= 1,
            _ => break,
        }

        stream.next();
    }

    let length = parse_length(stream);

    Ok(Some(Instruction::Note(pitch, length)))
}

pub fn play_pcm(stream: &mut RollbackableTokenStream) -> ParseResult {
    if stream.expect_character('p').is_err() {
        return Ok(None);
    }

    let params = stream.comma_separated_n_numbers(2)?;

    let pcm_number = params[0];
    let sampling_rate = params[1] as f32;

    Ok(Some(Instruction::PlayPCM(pcm_number, sampling_rate)))
}
