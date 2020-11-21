#[derive(Debug)]
struct FixedLengthQueue<T: Copy> {
    elements: Vec<T>,
    cursor: usize,
}

impl<T: Copy> FixedLengthQueue<T> {
    pub fn new(length: usize, default: T) -> Self {
        FixedLengthQueue {
            elements: vec![default; length],
            cursor: 0,
        }
    }

    pub fn push(&mut self, element: T) -> T {
        let popped = self.elements[self.cursor];
        self.elements[self.cursor] = element;
        self.cursor = (self.cursor + 1) % self.elements.len();
        popped
    }

    pub fn peek(&mut self) -> T {
        self.elements[self.cursor]
    }

    pub fn modify<F: FnOnce(T) -> T>(&mut self, modifier: F) -> T {
        let modified = modifier(self.peek());
        self.push(modified);
        modified
    }
}

pub trait Effector: std::fmt::Debug + Send + Sync {
    fn apply(&mut self, sample: f32) -> f32;
}

#[derive(Debug)]
pub struct Delay {
    feedback: f32,
    delayed: FixedLengthQueue<f32>,
}

impl Delay {
    pub fn new(delay_sec: f32, feedback: f32, sample_rate: f32) -> Self {
        let delay_samples = (delay_sec * sample_rate) as usize;

        Delay {
            feedback,
            delayed: FixedLengthQueue::new(delay_samples, 0.0),
        }
    }
}

impl Effector for Delay {
    fn apply(&mut self, sample: f32) -> f32 {
        let feedback = self.feedback;
        self.delayed.modify(|x| x * feedback + sample)
    }
}

#[derive(Debug)]
pub struct EffectsQueue {
    effects: Vec<(f32, Box<dyn Effector>)>,
}

impl EffectsQueue {
    pub fn new(mut effects: Vec<(f32, Box<dyn Effector>)>) -> Self {
        effects.sort_unstable_by(|(a, _), (b, _)| b.partial_cmp(a).unwrap());
        EffectsQueue { effects }
    }

    pub fn next_before(&mut self, before: f32) -> Option<Box<dyn Effector>> {
        if self.effects.last()?.0 > before {
            return None;
        }

        Some(self.effects.remove(self.effects.len() - 1).1)
    }

    pub fn iter(&self) -> impl Iterator<Item=&(f32, Box<dyn Effector>)> {
        self.effects.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }
}
