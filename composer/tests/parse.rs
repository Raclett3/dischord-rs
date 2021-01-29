use composer::*;

#[test]
fn test_parse() {
    use parse::{parse, Instruction, NoteLength::*, ToneModifier};
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
                Instruction::ToneModifier(ToneModifier::Tone(2)),
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
        Ok(Some(Note(3, vec![Length(2), Dot])))
    );
    assert_eq!(
        single_parse(note, "E++C"),
        Ok(Some(Note(9, vec![DefaultLength])))
    );
    assert_eq!(single_parse(note, "H"), Ok(None));
}

#[test]
fn test_rest() {
    use parse::note::rest;
    use parse::{Instruction::Rest, NoteLength::*};

    assert_eq!(
        single_parse(rest, "R4.R8"),
        Ok(Some(Rest(vec![Length(4), Dot])))
    );
    assert_eq!(single_parse(rest, "C4"), Ok(None));
}

#[test]
fn test_length() {
    use parse::note::length;
    use parse::{Instruction, NoteLength::*};

    assert_eq!(
        single_parse(length, "L8..L9"),
        Ok(Some(Instruction::Length(vec![Length(8), Dot, Dot])))
    );
    assert_eq!(single_parse(length, "DEF"), Ok(None));
}

#[test]
fn test_octave() {
    use parse::octave::octave;
    use parse::Instruction::Octave;

    assert_eq!(single_parse(octave, "<"), Ok(Some(Octave(1))));
    assert_eq!(single_parse(octave, ">"), Ok(Some(Octave(-1))));
    assert_eq!(single_parse(octave, "!"), Ok(None));
}

#[test]
fn test_tempo() {
    use parse::tempo::tempo;
    use parse::Instruction::Tempo;

    assert_eq!(single_parse(tempo, "T120"), Ok(Some(Tempo(120))));
    assert!(single_parse(tempo, "TA").is_err());
    assert!(single_parse(tempo, "T").is_err());
    assert!(single_parse(tempo, "A").unwrap().is_none());
}

#[test]
fn test_volume() {
    use parse::volume::volume;
    use parse::Instruction::Volume;

    assert_eq!(single_parse(volume, "V200"), Ok(Some(Volume(2.0))));
    assert!(single_parse(volume, "VB").is_err());
    assert!(single_parse(volume, "V").is_err());
    assert!(single_parse(volume, "C").unwrap().is_none());
}

#[test]
fn test_tone() {
    use parse::tone::tone;
    use parse::{
        Instruction::ToneModifier,
        ToneModifier::{DefinePCMTone, Detune, Envelope, Tone},
    };

    assert_eq!(single_parse(tone, "@2"), Ok(Some(ToneModifier(Tone(2)))));
    assert_eq!(
        single_parse(tone, "@D2,10000"),
        Ok(Some(ToneModifier(Detune(2, 1.0))))
    );
    assert_eq!(
        single_parse(tone, "@E0,100,100,200"),
        Ok(Some(ToneModifier(Envelope(0.0, 1.0, 1.0, 2.0))))
    );
    assert_eq!(
        single_parse(tone, "@H{08F}"),
        Ok(Some(ToneModifier(DefinePCMTone(vec![
            -1.0,
            0.0,
            7.0 / 8.0
        ]))))
    );
    assert!(single_parse(tone, "@D3").is_err());
    assert!(single_parse(tone, "@D1,10,100").is_err());
    assert!(single_parse(tone, "@E1,10,100").is_err());
    assert!(single_parse(tone, "@E0,1,2,3,4").is_err());
    assert!(single_parse(tone, "@M").is_err());
    assert!(single_parse(tone, "@").is_err());
    assert!(single_parse(tone, "0").unwrap().is_none());
}

#[test]
fn test_chord() {
    use parse::note::chord;
    use parse::{Instruction::Chord, NoteLength::*};

    assert_eq!(
        single_parse(chord, "(CEG<C->C+)2"),
        Ok(Some(Chord(vec![3, 7, 10, 14, 4], vec![Length(2)])))
    );
    assert!(single_parse(chord, "(CE").is_err());
    assert!(single_parse(chord, "(CEH)").is_err());
    assert_eq!(single_parse(chord, "C4"), Ok(None));
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
        Ok(Some(Repeat(
            vec![
                Note(3, vec![DefaultLength]),
                Note(5, vec![DefaultLength]),
                Note(7, vec![DefaultLength])
            ],
            4
        )))
    );
    assert!(single_parse(repeat, "[CDE]").is_err());
    assert!(single_parse(repeat, "[CD;E]").is_err());
    assert!(single_parse(repeat, "[!?]").is_err());
    assert!(single_parse(repeat, "[CDE").is_err());
    assert!(single_parse(repeat, "94").unwrap().is_none());
}
