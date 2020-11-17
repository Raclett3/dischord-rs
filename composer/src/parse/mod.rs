pub mod note;
pub mod octave;
pub mod repeat;
pub mod tempo;
pub mod tone;
pub mod volume;

use crate::tokenize::{Token, TokenKind};
use std::fmt;

#[macro_export]
macro_rules! try_or_ok_none {
    ($option:expr) => {
        if let Ok(ok) = $option {
            ok
        } else {
            return Ok(None);
        }
    };
}

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
    Volume(f32),
    Gate(f32),
    Tone(usize),
    DefinePCMTone(Vec<f32>),
    PCMTone(usize),
    Detune(usize, f32),
    Envelope(f32, f32, f32, f32),
    Repeat(Track, usize),
    Length(Vec<NoteLength>),
    Tune(f32),
}

pub type Track = Vec<Instruction>;
pub type ParsedMML = Vec<Track>;

type ParseResult = Result<Option<Instruction>, ParseError>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError {
    UnexpectedToken(Token),
    WrongParamsNumber(usize, usize, usize), // Params at, expected, provided
    UnexpectedEOF,
}

impl ParseError {
    fn unexpected_char(position: usize, ch: char) -> Self {
        Self::UnexpectedToken((position, TokenKind::Character(ch)))
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedToken((token_at, token)) => {
                write!(f, "Unexpected token {} at {}", token, token_at)
            }
            ParseError::WrongParamsNumber(params_at, expected, provided) => write!(
                f,
                "{} parameter(s) are provided at {}, expected {} parameter(s)",
                provided, params_at, expected
            ),
            ParseError::UnexpectedEOF => write!(f, "Unexpected EOF"),
        }
    }
}

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

    pub fn take_number(&mut self) -> Result<(usize, usize), ParseError> {
        match self.peek() {
            Some(&(token_at, TokenKind::Number(num))) => {
                self.next();
                Ok((token_at, num))
            }
            Some(x) => Err(ParseError::UnexpectedToken(x.clone())),
            _ => Err(ParseError::UnexpectedEOF),
        }
    }

    pub fn take_character(&mut self) -> Result<(usize, char), ParseError> {
        match self.peek() {
            Some(&(token_at, TokenKind::Character(ch))) => {
                self.next();
                Ok((token_at, ch))
            }
            Some(x) => Err(ParseError::UnexpectedToken(x.clone())),
            _ => Err(ParseError::UnexpectedEOF),
        }
    }

    pub fn take_brace_string(&mut self) -> Result<(usize, &'a str), ParseError> {
        match self.peek() {
            Some((token_at, TokenKind::BraceString(string))) => {
                self.next();
                Ok((*token_at, string))
            }
            Some(x) => Err(ParseError::UnexpectedToken(x.clone())),
            _ => Err(ParseError::UnexpectedEOF),
        }
    }

    pub fn comma_separated_numbers(&mut self) -> Vec<usize> {
        let mut numbers = vec![];

        while let Ok((_, number)) = self.take_number() {
            numbers.push(number);
            if self.expect_character(',').is_err() {
                break;
            }
        }

        numbers
    }

    pub fn expect_character(&mut self, ch_a: char) -> Result<(), ParseError> {
        match self.peek() {
            Some(&(_, TokenKind::Character(ch_b))) if ch_a == ch_b => {
                self.next();
                Ok(())
            }
            Some(x) => Err(ParseError::UnexpectedToken(x.clone())),
            None => Err(ParseError::UnexpectedEOF),
        }
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn new(tokens: &'a [Token]) -> Self {
        RollbackableTokenStream { tokens, cursor: 0 }
    }
}

pub type Parser = fn(&mut RollbackableTokenStream) -> ParseResult;

pub fn parse_stream(
    stream: &mut RollbackableTokenStream,
    inside_bracket: bool,
) -> Result<ParsedMML, ParseError> {
    let mut parsed = Vec::new();
    let mut track = Vec::new();

    'main_loop: while !stream.empty() {
        if let Some(&(token_at, TokenKind::Character(';'))) = stream.peek() {
            stream.next();
            if inside_bracket {
                return Err(ParseError::unexpected_char(token_at, ';'));
            }
            stream.accept();
            parsed.push(track);
            track = Vec::new();
            continue;
        }

        if let Some(&(token_at, TokenKind::Character(']'))) = stream.peek() {
            stream.next();
            if inside_bracket {
                stream.accept();
                return Ok(vec![track]);
            } else {
                return Err(ParseError::unexpected_char(token_at, ']'));
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

        for &parser in &parsers {
            stream.rollback();
            if let Some(x) = parser(stream)? {
                track.push(x);
                stream.accept();
                continue 'main_loop;
            }
        }
        stream.rollback();
        let token = stream.next().unwrap();
        return Err(ParseError::UnexpectedToken(token.clone()));
    }

    if inside_bracket {
        return Err(ParseError::UnexpectedEOF);
    }

    if !track.is_empty() {
        parsed.push(track);
    }

    Ok(parsed)
}

pub fn parse(tokens: &[Token]) -> Result<ParsedMML, ParseError> {
    let mut stream = RollbackableTokenStream::new(tokens);
    parse_stream(&mut stream, false)
}
