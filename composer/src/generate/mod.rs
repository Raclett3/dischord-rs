pub mod note;

use crate::parse::NoteLength;

pub fn note_length_to_float(length: &[NoteLength], default: f64) -> f64 {
    length
        .iter()
        .scan(0.0, |last, x| {
            *last = match x {
                NoteLength::DefaultLength => default,
                NoteLength::Dot => *last / 2.0,
                NoteLength::Length(l) => 1.0 / (*l as f64),
            };
            Some(*last)
        })
        .sum()
}
