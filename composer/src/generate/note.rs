use crate::generate::Tone;

#[derive(Debug, PartialEq, Clone)]
pub struct Note {
    frequency: f32,
    tone: Tone,
    volume_from: f32,
    volume_to: f32,
    offset: f32,
    start_at: f32,
    end_at: f32,
}

impl Note {
    pub fn is_over(&self, position: f32) -> bool {
        self.end_at <= position
    }

    pub fn is_waiting(&self, position: f32) -> bool {
        position < self.start_at
    }

    pub fn is_ringing(&self, position: f32) -> bool {
        !self.is_over(position) && !self.is_waiting(position)
    }

    pub fn get_sample(&self, position: f32) -> f32 {
        if !self.is_ringing(position) {
            return 0.0;
        }

        let note_position = position - self.start_at;
        let progress_ratio = note_position / (self.end_at - self.start_at);
        let volume = self.volume_from + (self.volume_to - self.volume_from) * progress_ratio;
        self.tone
            .sample(self.frequency, self.offset + note_position)
            * volume
    }

    pub fn end_at(&self) -> f32 {
        self.end_at
    }

    pub fn new(
        frequency: f32,
        tone: Tone,
        volume_from: f32,
        volume_to: f32,
        offset: f32,
        start_at: f32,
        end_at: f32,
    ) -> Self {
        Self {
            frequency,
            tone,
            volume_from,
            volume_to,
            offset,
            start_at,
            end_at,
        }
    }
}

#[derive(Debug)]
pub struct NotesQueue {
    notes: Vec<Note>,
}

impl NotesQueue {
    pub fn new(mut notes: Vec<Note>) -> Self {
        notes.sort_unstable_by(|a, b| {
            b.start_at
                .partial_cmp(&a.start_at)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        NotesQueue { notes }
    }

    pub fn next_before(&mut self, before: f32) -> Option<Note> {
        if self.notes.last()?.is_waiting(before) {
            return None;
        }

        Some(self.notes.remove(self.notes.len() - 1))
    }

    pub fn iter(&self) -> impl Iterator<Item=&Note> {
        self.notes.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.notes.is_empty()
    }
}
