pub mod effects;
pub mod note;
pub mod tones;

use crate::parse::tone::Effect;
use crate::parse::{Instruction, NoteLength, Track};
use effects::{Effector, EffectsQueue};
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

pub fn parse_note<'a>(length: f32, pitch: isize, state: &mut TrackState<'a>) {
    let (attack, decay, sustain, release) = state.envelope;
    let (unison_count, detune) = state.detune;
    let mut frequency =
        220.0 * (2.0f32).powf((state.octave * 12 + pitch) as f32 / 12.0) * state.tune;
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
            state.notes.push(note);
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
            state.notes.push(note);
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
            state.notes.push(note);
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
            state.notes.push(note);
        }

        frequency *= 1.0 + detune;
    }
}

pub fn parse_instruction<'a>(inst: &Instruction, state: &mut TrackState<'a>) {
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
            parse_note(length, *pitch, state);
            state.position += length;
        }
        Instruction::Chord(pitch, length) => {
            let length = 240.0 / state.tempo * note_length_to_float(&length, state.default_length);
            for &note in pitch {
                parse_note(length, note, state);
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
                parse_track(track, state);
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
        Instruction::Effect(effect) => {
            let effect = match *effect {
                Effect::Delay { delay, feedback } => {
                    Box::new(effects::Delay::new(delay, feedback, state.sample_rate))
                }
            };
            state.effects.push((state.position, effect));
        }
    }
}

pub fn parse_track<'a>(track: &[Instruction], state: &mut TrackState<'a>) {
    for inst in track {
        parse_instruction(inst, state);
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
    sample_rate: f32,
    effects: Vec<(f32, Box<dyn Effector>)>,
    notes: Vec<Note>,
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
    pub fn new(sample_rate: f32, fn_tones: &'a [FnTone], pcm_tones: Vec<Arc<Vec<f32>>>) -> Self {
        Self {
            sample_rate,
            effects: Vec::new(),
            notes: Vec::new(),
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

    pub fn reset(&mut self) {
        self.position = 0.0;
        self.default_length = 1.0 / 8.0;
        self.volume = 1.0;
        self.tone = Tone::FnTone(self.fn_tones[0]);
        self.octave = 0;
        self.detune = (1, 0.0);
        self.envelope = (0.0, 0.0, 1.0, 0.0);
        self.gate = 0.001;
        self.tune = 1.0;
    }

    pub fn push_note(&mut self, note: Note) {
        self.notes.push(note);
    }

    pub fn drain_notes_queue(&mut self) -> NotesQueue {
        NotesQueue::new(self.notes.split_off(0))
    }

    pub fn drain_effects_queue(&mut self) -> EffectsQueue {
        EffectsQueue::new(self.effects.split_off(0))
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
    notes_queues: Vec<NotesQueue>,
    effects_queues: Vec<EffectsQueue>,
    ringing_notes: Vec<Vec<Note>>,
    applied_effects: Vec<Vec<Box<dyn Effector>>>,
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
        let mut state = TrackState::new(sample_rate, TONES, Vec::new());
        let (notes_queues, effects_queues): (Vec<_>, Vec<_>) = tracks
            .iter()
            .map(|track| {
                parse_track(track, &mut state);
                state.reset();
                (state.drain_notes_queue(), state.drain_effects_queue())
            })
            .unzip();

        let track_length = notes_queues
            .iter()
            .flat_map(|queue| queue.iter().map(|note| note.end_at()))
            .fold(0.0, partial_max);

        Self {
            sample_rate,
            position: 0.0,
            notes_queues,
            effects_queues,
            ringing_notes: vec![Vec::new(); tracks.len()],
            applied_effects: (0..(tracks.len())).map(|_| Vec::new()).collect(),
            track_length,
        }
    }

    pub fn is_over(&self) -> bool {
        self.track_length + 1.0 <= self.position
    }

    pub fn track_length(&self) -> f32 {
        self.track_length
    }

    pub fn into_i16_stream(self) -> impl Iterator<Item = i16> {
        self.map(|sample| (sample * 32767.0) as i16)
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
            .chain(self.into_i16_stream().flat_map(i16_to_bytes))
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

        let zipped = self
            .effects_queues
            .iter_mut()
            .zip(self.applied_effects.iter_mut());

        for (effects_queue, applied_effects) in zipped {
            while let Some(note) = effects_queue.next_before(self.position) {
                applied_effects.push(note);
            }
        }

        let zipped = self
            .notes_queues
            .iter_mut()
            .zip(self.ringing_notes.iter_mut())
            .zip(self.applied_effects.iter_mut());

        for ((notes_queue, ringing_notes), applied_effects) in zipped {
            while let Some(note) = notes_queue.next_before(self.position) {
                ringing_notes.push(note);
            }
            let mut cursor = 0;
            while cursor < ringing_notes.len() {
                if ringing_notes[cursor].is_over(self.position) {
                    ringing_notes.remove(cursor);
                } else {
                    cursor += 1;
                }
            }
            let mut track_sample = 0.0;
            for note in ringing_notes {
                track_sample += note.get_sample(self.position);
            }

            for effect in applied_effects {
                track_sample = effect.apply(track_sample);
            }

            sample += track_sample
        }

        self.position += 1.0 / self.sample_rate;

        sample = partial_max(-1.0, partial_min(sample / 4.0, 1.0));

        Some(sample)
    }
}
