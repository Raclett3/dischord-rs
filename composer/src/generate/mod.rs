use crate::parse::NoteLength;

pub fn note_length_to_float(length: Vec<NoteLength>, default: usize) -> f64 {
    length
        .iter()
        .scan(default, |last, x| {
            *last = match x {
                NoteLength::DefaultLength => default,
                NoteLength::Dot => *last * 2,
                NoteLength::Length(l) => *l,
            };
            Some(*last)
        })
        .map(|x| 1.0 / (x as f64))
        .sum()
}
