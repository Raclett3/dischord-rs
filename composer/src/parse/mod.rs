pub mod note;
pub mod octave;
pub mod tempo;

use crate::tokenize::Token;

#[derive(Eq, PartialEq, Debug)]
pub enum NoteLength {
    DefaultLength,
    Dot,
    Length(usize),
}

#[derive(Eq, PartialEq, Debug)]
pub enum Instruction {
    Note(isize, Vec<NoteLength>),
    Rest(Vec<NoteLength>),
    Octave(isize),
    Tempo(usize),
}

pub type ParsedMML = Vec<Instruction>;

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

    pub fn new(tokens: &'a [Token]) -> Self {
        RollbackableTokenStream { tokens, cursor: 0 }
    }
}

pub fn parse(tokens: &[Token]) -> Result<ParsedMML, String> {
    let mut stream = RollbackableTokenStream::new(tokens);
    let mut parsed = Vec::new();

    while !stream.empty() {
        let parsers: [fn(&mut RollbackableTokenStream) -> ParseResult; 1] = [note::note];
        let result = parsers
            .iter()
            .map(|parser| {
                stream.rollback();
                parser(&mut stream)
            })
            .find(|x| x.is_some())
            .flatten();

        match result {
            None => {
                stream.rollback();
                let (token_at, token) = stream.next().unwrap();
                return Err(format!("Unexpected token {} at {}", token_at, token));
            }
            Some(Err(x)) => return Err(x),
            Some(Ok(x)) => parsed.push(x),
        }

        stream.accept();
    }

    Ok(parsed)
}
