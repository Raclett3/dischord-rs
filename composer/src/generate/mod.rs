pub mod note;
pub mod tones;

use crate::parse::{Instruction, NoteLength, Track};
use note::{Note, NotesQueue};
use std::sync::Arc;

pub type FnTone = fn(f32, f32) -> f32;

pub fn note_length_to_float(length: &[NoteLength], default: f32) -> f32 {
    length
        .iter()
        .scan(0.0, |last, x| {
            *last = match x {
                NoteLength::DefaultLength => default,
                NoteLength::Dot => *last / 2.0,
                NoteLength::Length(l) => 1.0 / (*l as f32),
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

pub fn parse_note<'a>(length: f32, pitch: isize, state: &TrackState<'a>, notes: &mut Vec<Note>) {
    let (attack, decay, sustain, release) = state.envelope;
    let (unison_count, detune) = state.detune;
    let mut frequency = 220.0 * (2.0f32).powf((state.octave * 12 + pitch) as f32 / 12.0) * state.tune;
    let length = partial_max(length - state.gate, 0.0);

    for _ in 0..unison_count {
        if state.envelope.0 != 0.0 {
            let attack_len = partial_min(length, attack);
            let note = Note::new(
                frequency,
                state.tone.clone(),
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
                state.tone.clone(),
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
                state.tone.clone(),
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
                let decay_length = length - attack;
                state.volume - (state.volume - state.volume * sustain) * decay_length / decay
            } else {
                sustain_volume
            };

            let release_len = release;
            let note = Note::new(
                frequency,
                state.tone.clone(),
                init_volume,
                0.0,
                length,
                state.position + length,
                state.position + length + release_len,
            );
            notes.push(note);
        }

        frequency *= 1.0 + detune;
    }
}

pub fn parse_instruction<'a>(
    inst: &Instruction,
    state: &mut TrackState<'a>,
    notes: &mut Vec<Note>,
) {
    match inst {
        Instruction::Octave(octave) => state.octave += octave,
        Instruction::Tempo(tempo) => state.tempo = *tempo as f32,
        Instruction::Volume(volume) => state.volume = *volume as f32,
        Instruction::Tone(tone) => {
            state.tone = Tone::FnTone(*state.fn_tones.get(*tone).unwrap_or(&state.fn_tones[0]))
        }
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
        Instruction::DefinePCMTone(pcm) => {
            state.pcm_tones.push(Arc::new(pcm.clone()));
        }
        Instruction::PCMTone(pcm) => {
            state.tone = if *pcm < state.pcm_tones.len() {
                Tone::PCMTone(state.pcm_tones[*pcm].clone())
            } else {
                Tone::FnTone(state.fn_tones[0])
            };
        }
        Instruction::Gate(gate) => state.gate = *gate,
        Instruction::Tune(tune) => state.tune = *tune,
    }
}

pub fn parse_track<'a>(track: &[Instruction], state: &mut TrackState<'a>, notes: &mut Vec<Note>) {
    for inst in track {
        parse_instruction(inst, state, notes);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Tone {
    FnTone(FnTone),
    PCMTone(Arc<Vec<f32>>),
}

impl Tone {
    pub fn sample(&self, frequency: f32, position: f32) -> f32 {
        match self {
            Tone::FnTone(tone) => tone(frequency, position),
            Tone::PCMTone(tone) => {
                let len = tone.len() as f32;
                let index = ((frequency * position * len) % len) as usize;
                tone[index]
            }
        }
    }
}

pub struct TrackState<'a> {
    position: f32,
    tempo: f32,
    default_length: f32,
    volume: f32,
    tone: Tone,
    fn_tones: &'a [FnTone],
    octave: isize,
    detune: (usize, f32),
    envelope: (f32, f32, f32, f32),
    gate: f32,
    tune: f32,
    pcm_tones: Vec<Arc<Vec<f32>>>,
}

impl<'a> TrackState<'a> {
    pub fn new(fn_tones: &'a [FnTone], pcm_tones: Vec<Arc<Vec<f32>>>) -> Self {
        Self {
            position: 0.0,
            tempo: 120.0,
            default_length: 1.0 / 8.0,
            volume: 1.0,
            tone: Tone::FnTone(fn_tones[0]),
            fn_tones,
            octave: 0,
            detune: (1, 0.0),
            envelope: (0.0, 0.0, 1.0, 0.0),
            gate: 0.001,
            tune: 1.0,
            pcm_tones,
        }
    }
}

fn u16_to_bytes(value: u16) -> impl Iterator<Item = u8> {
    (0..2).map(move |x| (value >> (x * 8)) as u8)
}

fn i16_to_bytes(value: i16) -> impl Iterator<Item = u8> {
    (0..2).map(move |x| (value >> (x * 8)) as u8)
}

fn u32_to_bytes(value: u32) -> impl Iterator<Item = u8> {
    (0..4).map(move |x| (value >> (x * 8)) as u8)
}

#[derive(Debug)]
pub struct Generator {
    sample_rate: f32,
    position: f32,
    notes_queue: NotesQueue,
    ringing_notes: Vec<Note>,
    track_length: f32,
}

static TONES: &[FnTone] = &[
    tones::pulse50,
    tones::pulse25,
    tones::pulse125,
    tones::triangle,
    tones::saw,
    tones::sine,
    tones::noise,
];

impl Generator {
    pub fn new(sample_rate: f32, tracks: &[Track]) -> Self {
        let mut notes = Vec::new();
        let mut tempo = 120.0;
        let mut pcm_tones = Vec::new();
        for track in tracks {
            let mut state = TrackState::new(TONES, pcm_tones);
            state.tempo = tempo;
            parse_track(track, &mut state, &mut notes);
            tempo = state.tempo;
            pcm_tones = state.pcm_tones;
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

    pub fn track_length(&self) -> f32 {
        self.track_length
    }

    pub fn into_riff(self) -> Vec<u8> {
        let sample_rate = self.sample_rate as u32;
        let channels = 1;
        let bits_per_sample = 16;
        let block_align = channels * bits_per_sample / 8u16;
        let byte_rate = sample_rate * block_align as u32;

        let mut riff: Vec<_> = std::iter::empty()
            // RIFF Chunk
            .chain(b"RIFF".iter().copied())
            .chain(u32_to_bytes(0)) // Chunk Size: Overwrite later
            .chain(b"WAVE".iter().copied())
            // Format Subchunk
            .chain(b"fmt ".iter().copied()) // Subchunk ID
            .chain(u32_to_bytes(16)) // Subchunk Size
            .chain(u16_to_bytes(1)) // PCM
            .chain(u16_to_bytes(channels as u16))
            .chain(u32_to_bytes(sample_rate))
            .chain(u32_to_bytes(byte_rate))
            .chain(u16_to_bytes(block_align))
            .chain(u16_to_bytes(bits_per_sample))
            .chain(b"data".iter().copied())
            .chain(u32_to_bytes(0)) // Data Size: Overwrite later
            .chain(self.flat_map(|sample| i16_to_bytes((sample * 32767.0) as i16)))
            .collect();

        for (i, byte) in (4..=7).zip(u32_to_bytes(riff.len() as u32)) {
            riff[i] = byte;
        }

        for (i, byte) in (40..=43).zip(u32_to_bytes(riff.len() as u32 - 44)) {
            riff[i] = byte;
        }

        riff
    }
}

impl Iterator for Generator {
    type Item = f32;

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
