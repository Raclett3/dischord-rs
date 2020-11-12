pub mod note;
pub mod tones;

use crate::parse::{Instruction, NoteLength, Track};
use note::{Note, NotesQueue};

pub type Tone = fn(f64, f64) -> f64;

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

fn partial_min<T: Copy + PartialOrd>(a: T, b: T) -> T {
    if a < b {
        a
    } else {
        b
    }
}

fn partial_max<T: Copy + PartialOrd>(a: T, b: T) -> T {
    if a > b {
        a
    } else {
        b
    }
}

pub fn parse_note(length: f64, pitch: isize, state: &TrackState, notes: &mut Vec<Note>) {
    let (attack, decay, sustain, release) = state.envelope;
    let frequency = 220.0 * (2.0f64).powf((state.octave * 12 + pitch) as f64 / 12.0);
    if state.envelope.0 != 0.0 {
        let attack_len = partial_min(length, attack);
        let note = Note::new(
            frequency,
            state.tone,
            0.0,
            state.volume * attack_len / attack,
            0.0,
            state.position,
            state.position + attack_len,
        );
        notes.push(note);
    }

    if decay > 0.0 && length > attack {
        let decay_len = partial_min(length - attack, decay);
        let note = Note::new(
            frequency,
            state.tone,
            state.volume,
            state.volume - (state.volume - state.volume * sustain) * decay_len / decay,
            attack,
            state.position + attack,
            state.position + attack + decay_len,
        );
        notes.push(note);
    }

    if length > attack + decay {
        let sustain_len = length - (attack + decay);
        let note = Note::new(
            frequency,
            state.tone,
            state.volume * sustain,
            state.volume * sustain,
            attack + decay,
            state.position + attack + decay,
            state.position + attack + decay + sustain_len,
        );
        notes.push(note);
    }

    if release > 0.0 {
        let sustain_volume = state.volume * sustain;

        let init_volume = if length < attack {
            length / attack * state.volume
        } else if length < attack + decay {
            let decay_length = length - decay;
            state.volume - (state.volume - state.volume * sustain) * decay_length / decay
        } else {
            sustain_volume
        };

        let release_len = release * init_volume / sustain_volume;
        let note = Note::new(
            frequency,
            state.tone,
            init_volume,
            0.0,
            length,
            state.position + length,
            state.position + length + release_len,
        );
        notes.push(note);
    }
}

pub fn parse_instruction(inst: &Instruction, state: &mut TrackState, notes: &mut Vec<Note>) {
    match inst {
        Instruction::Octave(octave) => state.octave += octave,
        Instruction::Tempo(tempo) => state.tempo = *tempo as f64,
        Instruction::Volume(volume) => state.volume = *volume as f64,
        Instruction::Tone(tone) => state.tone = *state.tones.get(*tone).unwrap_or(&state.tones[0]),
        Instruction::Detune(number, ratio) => state.detune = (*number, *ratio),
        Instruction::Envelope(a, d, s, r) => state.envelope = (*a, *d, *s, *r),
        Instruction::Note(pitch, length) => {
            let length = 240.0 / state.tempo * note_length_to_float(&length, state.default_length);
            parse_note(length, *pitch, state, notes);
            state.position += length;
        }
        Instruction::Chord(pitch, length) => {
            let length = 240.0 / state.tempo * note_length_to_float(&length, state.default_length);
            for &note in pitch {
                parse_note(length, note, state, notes);
            }
            state.position += length;
        }
        Instruction::Rest(length) => {
            let length = 240.0 / state.tempo * note_length_to_float(&length, state.default_length);
            state.position += length;
        }
        Instruction::Length(length) => {
            state.default_length = note_length_to_float(&length, state.default_length);
        }
        Instruction::Repeat(track, times) => {
            for _ in 0..*times {
                parse_track(track, state, notes);
            }
        }
    }
}

pub fn parse_track(track: &[Instruction], state: &mut TrackState, notes: &mut Vec<Note>) {
    for inst in track {
        parse_instruction(inst, state, notes);
    }
}

pub struct TrackState<'a> {
    position: f64,
    tempo: f64,
    default_length: f64,
    volume: f64,
    tone: Tone,
    tones: &'a [Tone],
    octave: isize,
    detune: (usize, f64),
    envelope: (f64, f64, f64, f64),
}

impl<'a> TrackState<'a> {
    pub fn new(tones: &'a [Tone]) -> Self {
        Self {
            position: 0.0,
            tempo: 120.0,
            default_length: 1.0 / 8.0,
            volume: 1.0,
            tone: tones[0],
            tones,
            octave: 0,
            detune: (1, 0.0),
            envelope: (0.0, 0.0, 1.0, 0.0),
        }
    }
}

#[derive(Debug)]
pub struct Generator {
    sample_rate: f64,
    position: f64,
    notes_queue: NotesQueue,
    ringing_notes: Vec<Note>,
    track_length: f64,
}

static TONES: &[Tone] = &[
    tones::pulse50,
    tones::pulse25,
    tones::pulse125,
    tones::triangle,
    tones::saw,
    tones::sine,
    tones::noise,
];

impl Generator {
    pub fn new(sample_rate: f64, tracks: &[Track]) -> Self {
        let mut notes = Vec::new();
        let mut tempo = 120.0;
        for track in tracks {
            let mut state = TrackState::new(TONES);
            state.tempo = tempo;
            parse_track(track, &mut state, &mut notes);
            tempo = state.tempo;
        }

        let track_length = notes
            .iter()
            .map(|note| note.end_at())
            .fold(0.0, partial_max);

        Self {
            sample_rate,
            position: 0.0,
            notes_queue: NotesQueue::new(notes),
            ringing_notes: vec![],
            track_length,
        }
    }

    pub fn is_over(&self) -> bool {
        self.ringing_notes.is_empty() && self.notes_queue.is_empty()
    }

    pub fn track_length(&self) -> f64 {
        self.track_length
    }
}

impl Iterator for Generator {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_over() {
            return None;
        }

        let mut sample = 0.0;
        while let Some(note) = self.notes_queue.next_before(self.position) {
            self.ringing_notes.push(note);
        }

        let mut cursor = 0;

        while cursor < self.ringing_notes.len() {
            if self.ringing_notes[cursor].is_over(self.position) {
                self.ringing_notes.remove(cursor);
            } else {
                cursor += 1;
            }
        }

        for note in &self.ringing_notes {
            sample += note.get_sample(self.position);
        }

        self.position += 1.0 / self.sample_rate;

        sample = partial_max(-1.0, partial_min(sample / 4.0, 1.0));

        Some(sample)
    }
}
