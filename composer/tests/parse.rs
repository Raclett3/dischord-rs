use composer::*;

#[test]
fn test_parse() {
    use parse::{parse, Instruction, NoteLength::*};
    use tokenize::tokenize;
    assert_eq!(
        parse(&tokenize("T150ab8r4&8..<c4;(cde)4@2[c4d4]2").unwrap()).unwrap(),
        vec![
            vec![
                Instruction::Tempo(150),
                Instruction::Note(12, vec![DefaultLength]),
                Instruction::Note(14, vec![Length(8)]),
                Instruction::Rest(vec![Length(4), Length(8), Dot, Dot]),
                Instruction::Octave(1),
                Instruction::Note(3, vec![Length(4)])
            ],
            vec![
                Instruction::Chord(vec![3, 5, 7], vec![Length(4)]),
                Instruction::Tone(2),
                Instruction::Repeat(
                    vec![
                        Instruction::Note(3, vec![Length(4)]),
                        Instruction::Note(5, vec![Length(4)])
                    ],
                    2
                )
            ]
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
    use parse::{Instruction, NoteLength::*};

    assert_eq!(
        single_parse(length, "L8..L9"),
        Some(Ok(Instruction::Length(vec![Length(8), Dot, Dot])))
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

    assert_eq!(single_parse(volume, "V200"), Some(Ok(Volume(2.0))));
    assert!(single_parse(volume, "VB").unwrap().is_err());
    assert!(single_parse(volume, "V").unwrap().is_err());
    assert!(single_parse(volume, "C").is_none());
}

#[test]
fn test_tone() {
    use parse::tone::tone;
    use parse::Instruction::{Detune, Envelope, Tone};

    assert_eq!(single_parse(tone, "@2"), Some(Ok(Tone(2))));
    assert_eq!(single_parse(tone, "@D2,100"), Some(Ok(Detune(2, 1.0))));
    assert!(single_parse(tone, "@D3").unwrap().is_err());
    assert!(single_parse(tone, "@D1,10,100").unwrap().is_err());
    assert_eq!(
        single_parse(tone, "@E0,100,100,200"),
        Some(Ok(Envelope(0.0, 1.0, 1.0, 2.0)))
    );
    assert!(single_parse(tone, "@D3").unwrap().is_err());
    assert!(single_parse(tone, "@D1,10,100").unwrap().is_err());
    assert!(single_parse(tone, "@E1,10,100").unwrap().is_err());
    assert!(single_parse(tone, "@E0,1,2,3,4").unwrap().is_err());
    assert!(single_parse(tone, "@M").unwrap().is_err());
    assert!(single_parse(tone, "@").unwrap().is_err());
    assert!(single_parse(tone, "0").is_none());
}

#[test]
fn test_chord() {
    use parse::note::chord;
    use parse::{Instruction::Chord, NoteLength::*};

    assert_eq!(
        single_parse(chord, "(CEG<C>C)2"),
        Some(Ok(Chord(vec![3, 7, 10, 15, 3], vec![Length(2)])))
    );
    assert!(single_parse(chord, "(CE").unwrap().is_err());
    assert!(single_parse(chord, "(CEH)").unwrap().is_err());
    assert_eq!(single_parse(chord, "C4"), None);
}

#[test]
fn test_repeat() {
    use parse::repeat::repeat;
    use parse::{
        Instruction::{Note, Repeat},
        NoteLength::DefaultLength,
    };

    assert_eq!(
        single_parse(repeat, "[CDE]4"),
        Some(Ok(Repeat(
            vec![
                Note(3, vec![DefaultLength]),
                Note(5, vec![DefaultLength]),
                Note(7, vec![DefaultLength])
            ],
            4
        )))
    );
    assert!(single_parse(repeat, "[CDE]").unwrap().is_err());
    assert!(single_parse(repeat, "[CD;E]").unwrap().is_err());
    assert!(single_parse(repeat, "[!?]").unwrap().is_err());
    assert!(single_parse(repeat, "[CDE").unwrap().is_err());
    assert!(single_parse(repeat, "94").is_none());
}
