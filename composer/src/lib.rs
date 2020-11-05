pub mod parse;
pub mod tokenize;

pub fn hello() {
    println!("Hello, world");
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn token_test() {
        use tokenize::TokenKind;

        assert!(TokenKind::Character(b'c').is_character());
        assert!(TokenKind::Number(42).is_number());
    }

    #[test]
    fn tokenize_test() {
        use tokenize::*;
        use TokenKind::*;
        assert!(tokenize("Do some 焼き松茸").is_err());
        assert!(tokenize("9999999999999999999999999999999999999999999999999").is_err());
        assert_eq!(
            tokenize("c256e16g4<CEG4"),
            Ok(vec![
                (1, Character(b'c')),
                (2, Number(256)),
                (5, Character(b'e')),
                (6, Number(16)),
                (8, Character(b'g')),
                (9, Number(4)),
                (10, Character(b'<')),
                (11, Character(b'c')),
                (12, Character(b'e')),
                (13, Character(b'g')),
                (14, Number(4)),
            ])
        );
        assert_eq!(
            tokenize("C e\n\rG"),
            Ok(vec![
                (1, Character(b'c')),
                (3, Character(b'e')),
                (6, Character(b'g')),
            ])
        );
    }

    #[test]
    fn test_length() {
        use parse::note::length;
        use parse::{NoteLength::*, RollbackableTokenStream};
        use tokenize::tokenize;

        let tokens = tokenize("123..&45&A13.&2").unwrap();
        let mut stream = RollbackableTokenStream::new(&tokens);
        assert_eq!(
            length(&mut stream),
            vec![Length(123), Dot, Dot, Length(45), DefaultLength]
        );

        let tokens = tokenize("C").unwrap();
        let mut stream = RollbackableTokenStream::new(&tokens);
        assert_eq!(length(&mut stream), vec![DefaultLength]);
    }

    #[test]
    fn test_note() {
        use parse::note::note;
        use parse::{Instruction::Note, NoteLength::*, RollbackableTokenStream};
        use tokenize::tokenize;

        let tokens = tokenize("C2.C4").unwrap();
        let mut stream = RollbackableTokenStream::new(&tokens);
        assert_eq!(note(&mut stream), Some(Ok(Note(3, vec![Length(2), Dot]))));

        let tokens = tokenize("E++C").unwrap();
        let mut stream = RollbackableTokenStream::new(&tokens);
        assert_eq!(note(&mut stream), Some(Ok(Note(9, vec![DefaultLength]))));

        let tokens = tokenize("H").unwrap();
        let mut stream = RollbackableTokenStream::new(&tokens);
        assert_eq!(note(&mut stream), None);
    }

    #[test]
    fn test_rest() {
        use parse::note::rest;
        use parse::{Instruction::Rest, NoteLength::*, RollbackableTokenStream};
        use tokenize::tokenize;

        let tokens = tokenize("R4.R8").unwrap();
        let mut stream = RollbackableTokenStream::new(&tokens);
        assert_eq!(rest(&mut stream), Some(Ok(Rest(vec![Length(4), Dot]))));

        let tokens = tokenize("C4").unwrap();
        let mut stream = RollbackableTokenStream::new(&tokens);
        assert_eq!(rest(&mut stream), None);
    }

    #[test]
    fn test_octave() {
        use parse::octave::octave;
        use parse::{Instruction::Octave, RollbackableTokenStream};
        use tokenize::tokenize;

        let tokens = tokenize("<").unwrap();
        let mut stream = RollbackableTokenStream::new(&tokens);
        assert_eq!(octave(&mut stream), Some(Ok(Octave(1))));

        let tokens = tokenize(">").unwrap();
        let mut stream = RollbackableTokenStream::new(&tokens);
        assert_eq!(octave(&mut stream), Some(Ok(Octave(-1))));

        let tokens = tokenize("!").unwrap();
        let mut stream = RollbackableTokenStream::new(&tokens);
        assert_eq!(octave(&mut stream), None);
    }

    #[test]
    fn test_tempo() {
        use parse::tempo::tempo;
        use parse::{Instruction::Tempo, RollbackableTokenStream};
        use tokenize::tokenize;

        let tokens = tokenize("T120").unwrap();
        let mut stream = RollbackableTokenStream::new(&tokens);
        assert_eq!(tempo(&mut stream), Some(Ok(Tempo(120))));

        let tokens = tokenize("TA").unwrap();
        let mut stream = RollbackableTokenStream::new(&tokens);
        assert!(tempo(&mut stream).unwrap().is_err());

        let tokens = tokenize("T").unwrap();
        let mut stream = RollbackableTokenStream::new(&tokens);
        assert!(tempo(&mut stream).unwrap().is_err());
    }
}
