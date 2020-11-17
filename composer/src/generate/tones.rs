struct RNG(u64);

impl RNG {
    pub fn next(&mut self) -> u64 {
        let RNG(mut x) = self;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        *self = RNG(x);
        x
    }
}

use std::f32::consts::PI;

pub fn pulse50(frequency: f32, position: f32) -> f32 {
    if frequency * position % 1.0 < 0.5 {
        1.0
    } else {
        -1.0
    }
}

pub fn pulse25(frequency: f32, position: f32) -> f32 {
    if frequency * position % 1.0 < 0.5 {
        1.0
    } else {
        -1.0
    }
}

pub fn pulse125(frequency: f32, position: f32) -> f32 {
    if frequency * position % 1.0 < 0.125 {
        1.0
    } else {
        -1.0
    }
}

pub fn triangle(f: f32, t: f32) -> f32 {
    2.0 / PI * (2.0 * PI * f * t).sin().asin()
}

pub fn saw(f: f32, t: f32) -> f32 {
    2.0 * (f * (t + 0.5) % 1.0) - 1.0
}

pub fn sine(f: f32, t: f32) -> f32 {
    (2.0 * PI * f * t).sin()
}

static mut NOISE_RNG: RNG = RNG(12345);

pub fn noise(_: f32, _: f32) -> f32 {
    unsafe { NOISE_RNG.next() as f32 / std::u64::MAX as f32 * 2.0 - 1.0 }
}
