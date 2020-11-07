pub mod note;
pub mod octave;
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
    Length(Vec<NoteLength>),
}

pub type Track = Vec<Instruction>;
pub type ParsedMML = Vec<Track>;

type ParseResult = Option<Result<Instruction, String>>;

pub struct RollbackableTokenStream<'a> {
    tokens: &'a [Token],
    cursor: usize,
}

impl Iterator for RollbackableTokenStream<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.tokens.get(self.cursor).copied();
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

    pub fn peek(&self) -> Option<Token> {
        self.tokens.get(self.cursor).copied()
    }

    pub fn empty(&self) -> bool {
        self.cursor >= self.tokens.len()
    }

    pub fn take_number(&mut self) -> Option<usize> {
        match self.peek() {
            Some((_, TokenKind::Number(num))) => {
                self.next();
                Some(num)
            }
            _ => None,
        }
    }

    pub fn take_character(&mut self) -> Option<char> {
        match self.peek() {
            Some((_, TokenKind::Character(ch))) => {
                self.next();
                Some(ch)
            }
            _ => None,
        }
    }

    pub fn comma_separated_numbers(&mut self) -> Vec<usize> {
        let mut numbers = vec![];

        while let Some(number) = self.take_number() {
            numbers.push(number);
            if !self.expect_character(',') {
                break;
            }
        }

        numbers
    }

    pub fn expect_character(&mut self, ch_a: char) -> bool {
        match self.peek() {
            Some((_, TokenKind::Character(ch_b))) => {
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

pub fn parse(tokens: &[Token]) -> Result<ParsedMML, String> {
    let mut stream = RollbackableTokenStream::new(tokens);
    let mut parsed = Vec::new();
    let mut track = Vec::new();

    while !stream.empty() {
        if stream.expect_character(';') {
            stream.accept();
            parsed.push(track);
            track = Vec::new();
            continue;
        }

        let parsers = [
            note::note,
            note::rest,
            note::chord,
            octave::octave,
            tempo::tempo,
            tone::tone,
            volume::volume,
        ];
        let result = parsers
            .iter()
            .map(|parser: &Parser| {
                stream.rollback();
                parser(&mut stream)
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

    if !track.is_empty() {
        parsed.push(track);
    }

    Ok(parsed)
}
