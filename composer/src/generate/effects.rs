use std::f32::consts::PI;

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
pub struct LowPassFilter {
    in1: f32,
    in2: f32,
    out1: f32,
    out2: f32,
    omega: f32,
    alpha: f32,
}

impl LowPassFilter {
    pub fn new(cut_off: f32, sample_rate: f32) -> Self {
        let omega = 2.0 * PI * cut_off / sample_rate;
        let alpha = omega.sin() / (2.0 / (2.0f32.sqrt() / 2.0));

        LowPassFilter {
            in1: 0.0,
            in2: 0.0,
            out1: 0.0,
            out2: 0.0,
            omega,
            alpha,
        }
    }
}

impl Effector for LowPassFilter {
    fn apply(&mut self, sample: f32) -> f32 {
        let a0 = 1.0 + self.alpha;
        let a1 = -2.0 * self.omega.cos();
        let a2 = 1.0 - self.alpha;
        let b0 = (1.0 - self.omega.cos()) / 2.0;
        let b1 = 1.0 - self.omega.cos();
        let b2 = (1.0 - self.omega.cos()) / 2.0;

        let output = b0 / a0 * sample + b1 / a0 * self.in1 + b2 / a0 * self.in2
            - a1 / a0 * self.out1
            - a2 / a0 * self.out2;
        
        self.in2 = self.in1;
        self.in1 = sample;
        self.out2 = self.out1;
        self.out1 = output;

        return output;
    }
}

#[derive(Debug)]
pub struct HighPassFilter {
    in1: f32,
    in2: f32,
    out1: f32,
    out2: f32,
    omega: f32,
    alpha: f32,
}

impl HighPassFilter {
    pub fn new(cut_off: f32, sample_rate: f32) -> Self {
        let omega = 2.0 * PI * cut_off / sample_rate;
        let alpha = omega.sin() / (2.0 / (2.0f32.sqrt() / 2.0));

        HighPassFilter {
            in1: 0.0,
            in2: 0.0,
            out1: 0.0,
            out2: 0.0,
            omega,
            alpha,
        }
    }
}

impl Effector for HighPassFilter {
    fn apply(&mut self, sample: f32) -> f32 {
        let a0 = 1.0 + self.alpha;
        let a1 = -2.0 * self.omega.cos();
        let a2 = 1.0 - self.alpha;
        let b0 = (1.0 + self.omega.cos()) / 2.0;
        let b1 = -(1.0 + self.omega.cos());
        let b2 = (1.0 + self.omega.cos()) / 2.0;

        let output = b0 / a0 * sample + b1 / a0 * self.in1 + b2 / a0 * self.in2
            - a1 / a0 * self.out1
            - a2 / a0 * self.out2;
        
        self.in2 = self.in1;
        self.in1 = sample;
        self.out2 = self.out1;
        self.out1 = output;

        return output;
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
