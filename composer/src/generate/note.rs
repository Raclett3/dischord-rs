type Tone = fn(f64, f64) -> f64;

#[derive(Debug, PartialEq, Clone)]
pub struct Note {
    frequency: f64,
    tone: Tone,
    volume_from: f64,
    volume_to: f64,
    start_at: f64,
    end_at: f64,
}

impl Note {
    pub fn is_over(&self, position: f64) -> bool {
        self.end_at <= position
    }

    pub fn is_waiting(&self, position: f64) -> bool {
        position < self.start_at
    }

    pub fn is_ringing(&self, position: f64) -> bool {
        !self.is_over(position) && !self.is_waiting(position)
    }

    pub fn get_sample(&self, position: f64) -> f64 {
        if !self.is_ringing(position) {
            return 0.0;
        }

        let note_position = position - self.start_at;
        let progress_ratio = note_position / (self.end_at - self.start_at);
        let volume = self.volume_from + (self.volume_to - self.volume_from) * progress_ratio;
        (self.tone)(self.frequency, note_position) * volume
    }

    pub fn new(
        frequency: f64,
        tone: Tone,
        volume_from: f64,
        volume_to: f64,
        start_at: f64,
        end_at: f64,
    ) -> Self {
        Self {
            frequency,
            tone,
            volume_from,
            volume_to,
            start_at,
            end_at,
        }
    }
}

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

    pub fn next_before(&mut self, before: f64) -> Option<Note> {
        if self.notes.last()?.is_waiting(before) {
            return None;
        }

        Some(self.notes.remove(self.notes.len() - 1))
    }
}
