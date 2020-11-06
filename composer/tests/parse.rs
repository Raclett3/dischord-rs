use composer::*;

#[test]
fn test_parse() {
    use parse::{parse, Instruction, NoteLength::*};
    use tokenize::tokenize;
    assert_eq!(
        parse(&tokenize("T150ab8r4&8..<c4").unwrap()).unwrap(),
        vec![
            Instruction::Tempo(150),
            Instruction::Note(12, vec![DefaultLength]),
            Instruction::Note(14, vec![Length(8)]),
            Instruction::Rest(vec![Length(4), Length(8), Dot, Dot]),
            Instruction::Octave(1),
            Instruction::Note(3, vec![Length(4)])
        ]
    );
}

fn single_parse<T>(parser: fn(&mut parse::RollbackableTokenStream) -> T, mml: &str) -> T {
    use parse::RollbackableTokenStream;
    use tokenize::tokenize;

    let tokens = tokenize(mml).unwrap();
    let mut stream = RollbackableTokenStream::new(&tokens);
    parser(&mut stream)
}

#[test]
fn test_parse_length() {
    use parse::note::parse_length;
    use parse::NoteLength::*;

    assert_eq!(
        single_parse(parse_length, "123..&45&A13.&2"),
        vec![Length(123), Dot, Dot, Length(45), DefaultLength]
    );
    assert_eq!(single_parse(parse_length, "C"), vec![DefaultLength]);
}

#[test]
fn test_note() {
    use parse::note::note;
    use parse::{Instruction::Note, NoteLength::*};

    assert_eq!(
        single_parse(note, "C2.C4"),
        Some(Ok(Note(3, vec![Length(2), Dot])))
    );
    assert_eq!(
        single_parse(note, "E++C"),
        Some(Ok(Note(9, vec![DefaultLength])))
    );
    assert_eq!(single_parse(note, "H"), None);
}

#[test]
fn test_rest() {
    use parse::note::rest;
    use parse::{Instruction::Rest, NoteLength::*};

    assert_eq!(
        single_parse(rest, "R4.R8"),
        Some(Ok(Rest(vec![Length(4), Dot])))
    );
    assert_eq!(single_parse(rest, "C4"), None);
}

#[test]
fn test_length() {
    use parse::note::length;
    use parse::{Instruction::Rest, NoteLength::*};

    assert_eq!(
        single_parse(length, "L8..L9"),
        Some(Ok(Rest(vec![Length(8), Dot, Dot])))
    );
    assert_eq!(single_parse(length, "DEF"), None);
}

#[test]
fn test_octave() {
    use parse::octave::octave;
    use parse::Instruction::Octave;

    assert_eq!(single_parse(octave, "<"), Some(Ok(Octave(1))));
    assert_eq!(single_parse(octave, ">"), Some(Ok(Octave(-1))));
    assert_eq!(single_parse(octave, "!"), None);
}

#[test]
fn test_tempo() {
    use parse::tempo::tempo;
    use parse::Instruction::Tempo;

    assert_eq!(single_parse(tempo, "T120"), Some(Ok(Tempo(120))));
    assert!(single_parse(tempo, "TA").unwrap().is_err());
    assert!(single_parse(tempo, "T").unwrap().is_err());
    assert!(single_parse(tempo, "A").is_none());
}

#[test]
fn test_volume() {
    use parse::volume::volume;
    use parse::Instruction::Volume;

    assert_eq!(single_parse(volume, "V120"), Some(Ok(Volume(120))));
    assert!(single_parse(volume, "VB").unwrap().is_err());
    assert!(single_parse(volume, "V").unwrap().is_err());
    assert!(single_parse(volume, "C").is_none());
}
