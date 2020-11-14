pub mod note;
pub mod octave;
pub mod repeat;
pub mod tempo;
pub mod tone;
pub mod volume;

use crate::tokenize::{Token, TokenKind};

#[derive(Eq, PartialEq, Debug)]
pub enum NoteLength {
    DefaultLength,
    Dot,
    Length(usize),
}

#[derive(PartialEq, Debug)]
pub enum Instruction {
    Note(isize, Vec<NoteLength>),
    Chord(Vec<isize>, Vec<NoteLength>),
    Rest(Vec<NoteLength>),
    Octave(isize),
    Tempo(usize),
    Volume(f64),
    Tone(usize),
    Detune(usize, f64),
    Envelope(f64, f64, f64, f64),
    Repeat(Track, usize),
    Length(Vec<NoteLength>),
}

pub type Track = Vec<Instruction>;
pub type ParsedMML = Vec<Track>;

type ParseResult = Option<Result<Instruction, String>>;

#[derive(Clone)]
pub struct RollbackableTokenStream<'a> {
    tokens: &'a [Token],
    cursor: usize,
}

impl<'a> Iterator for RollbackableTokenStream<'a> {
    type Item = &'a Token;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.tokens.get(self.cursor);
        self.cursor += 1;
        item
    }
}

impl<'a> RollbackableTokenStream<'a> {
    pub fn rollback(&mut self) {
        self.cursor = 0;
    }

    pub fn accept(&mut self) {
        self.tokens = &self.tokens[self.cursor..];
        self.cursor = 0;
    }

    pub fn peek(&self) -> Option<&'a Token> {
        self.tokens.get(self.cursor)
    }

    pub fn empty(&self) -> bool {
        self.cursor >= self.tokens.len()
    }

    pub fn take_number(&mut self) -> Option<(usize, usize)> {
        match self.peek() {
            Some(&(token_at, TokenKind::Number(num))) => {
                self.next();
                Some((token_at, num))
            }
            _ => None,
        }
    }

    pub fn take_character(&mut self) -> Option<(usize, char)> {
        match self.peek() {
            Some(&(token_at, TokenKind::Character(ch))) => {
                self.next();
                Some((token_at, ch))
            }
            _ => None,
        }
    }

    pub fn comma_separated_numbers(&mut self) -> Vec<usize> {
        let mut numbers = vec![];

        while let Some((_, number)) = self.take_number() {
            numbers.push(number);
            if !self.expect_character(',') {
                break;
            }
        }

        numbers
    }

    pub fn expect_character(&mut self, ch_a: char) -> bool {
        match self.peek() {
            Some(&(_, TokenKind::Character(ch_b))) => {
                if ch_a == ch_b {
                    self.next();
                }
                ch_a == ch_b
            }
            _ => false,
        }
    }

    pub fn new(tokens: &'a [Token]) -> Self {
        RollbackableTokenStream { tokens, cursor: 0 }
    }
}

pub type Parser = fn(&mut RollbackableTokenStream) -> ParseResult;

pub fn parse_stream(
    stream: &mut RollbackableTokenStream,
    inside_bracket: bool,
) -> Result<ParsedMML, String> {
    let mut parsed = Vec::new();
    let mut track = Vec::new();

    while !stream.empty() {
        if stream.expect_character(';') {
            if inside_bracket {
                return Err("Unexpected token ;".to_string());
            }
            stream.accept();
            parsed.push(track);
            track = Vec::new();
            continue;
        }

        if stream.expect_character(']') {
            if inside_bracket {
                stream.accept();
                return Ok(vec![track]);
            } else {
                return Err("Unexpected token ]".to_string());
            }
        }

        let parsers = [
            note::note,
            note::rest,
            note::chord,
            note::length,
            octave::octave,
            tempo::tempo,
            tone::tone,
            volume::volume,
            repeat::repeat,
        ];
        let result = parsers
            .iter()
            .map(|parser: &Parser| {
                stream.rollback();
                parser(stream)
            })
            .flatten()
            .next();

        match result {
            None => {
                stream.rollback();
                let (token_at, token) = stream.next().unwrap();
                return Err(format!("Unexpected token {} at {}", token_at, token));
            }
            Some(Err(x)) => return Err(x),
            Some(Ok(x)) => track.push(x),
        }

        stream.accept();
    }

    if inside_bracket {
        return Err("Unexpected EOF".to_string());
    }

    if !track.is_empty() {
        parsed.push(track);
    }

    Ok(parsed)
}

pub fn parse(tokens: &[Token]) -> Result<ParsedMML, String> {
    let mut stream = RollbackableTokenStream::new(tokens);
    parse_stream(&mut stream, false)
}
