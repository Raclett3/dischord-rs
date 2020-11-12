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

use std::f64::consts::PI;

pub fn pulse50(frequency: f64, position: f64) -> f64 {
    if frequency * position % 1.0 < 0.5 {
        1.0
    } else {
        -1.0
    }
}

pub fn pulse25(frequency: f64, position: f64) -> f64 {
    if frequency * position % 1.0 < 0.5 {
        1.0
    } else {
        -1.0
    }
}

pub fn pulse125(frequency: f64, position: f64) -> f64 {
    if frequency * position % 1.0 < 0.125 {
        1.0
    } else {
        -1.0
    }
}

pub fn triangle(f: f64, t: f64) -> f64 {
    2.0 / PI * (2.0 * PI * f * t).sin().asin()
}

pub fn saw(f: f64, t: f64) -> f64 {
    2.0 * (f * (t + 0.5) % 1.0) - 1.0
}

pub fn sine(f: f64, t: f64) -> f64 {
    (2.0 * PI * f * t).sin()
}

static mut NOISE_RNG: RNG = RNG(12345);

pub fn noise(_: f64, _: f64) -> f64 {
    unsafe { NOISE_RNG.next() as f64 / std::u64::MAX as f64 * 2.0 - 1.0 }
}
