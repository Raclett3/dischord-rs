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

use once_cell::sync::OnceCell;
use std::f32::consts::PI;
use std::sync::Mutex;

struct WaveCache {
    resolution: usize,
    wave: OnceCell<Mutex<Vec<Option<f32>>>>,
}

impl WaveCache {
    const fn new() -> Self {
        WaveCache {
            resolution: 3000,
            wave: OnceCell::new(),
        }
    }

    fn get_cache(&self) -> &Mutex<Vec<Option<f32>>> {
        self.wave
            .get_or_init(|| Mutex::new(vec![None; self.resolution]))
    }

    fn sample<F: Fn(f32, f32) -> f32>(&self, frequency: f32, position: f32, func: F) -> f32 {
        let mut cache = self.get_cache().lock().unwrap();
        let cache_position =
            (position * frequency * self.resolution as f32) as usize % self.resolution;

        if let Some(sample) = cache[cache_position] {
            return sample;
        }

        let sample = func(frequency, position);
        cache[cache_position] = Some(sample);
        sample
    }
}

/*
 * d = duty, n = nth overtone, f = frequency, x = time
 * y = PI(d - 1/2) + SUM(n=1..inf) 1/n(1 - cos2nPId)sin2nPIfx + 1/n(sin2nPId)cos2nPIfx
 */

fn pulse(duty: f32) -> impl Fn(f32, f32) -> f32 {
    move |frequency: f32, position: f32| {
        PI * (duty - 0.5)
            + (1..)
                .map(|x| x as f32)
                .take_while(|x| x * frequency < 20000.0)
                .map(|n| {
                    (1.0 - (2.0 * n * PI * duty).cos()) * (2.0 * n * PI * frequency * position).sin()
                        / n
                        + (2.0 * n * PI * duty).sin() * (2.0 * n * PI * frequency * position).cos()
                            / n
                })
                .sum::<f32>()
    }
}

pub fn pulse50(frequency: f32, position: f32) -> f32 {
    static CACHE: WaveCache = WaveCache::new();
    CACHE.sample(frequency, position, pulse(0.5))
}

pub fn pulse25(frequency: f32, position: f32) -> f32 {
    static CACHE: WaveCache = WaveCache::new();
    CACHE.sample(frequency, position, pulse(0.25))
}

pub fn pulse125(frequency: f32, position: f32) -> f32 {
    static CACHE: WaveCache = WaveCache::new();
    CACHE.sample(frequency, position, pulse(0.125))
}

pub fn triangle(f: f32, t: f32) -> f32 {
    2.0 / PI * (2.0 * PI * f * t).sin().asin()
}

/*
 * n = nth overtone, f = frequency, x = time
 * y = PI(d - 1/2) + SUM(n=1..inf) 1/n(1 - cos2nPId)sin2nPIfx + 1/n(sin2nPId)cos2nPIfx
 */

pub fn saw(frequency: f32, position: f32) -> f32 {
    static CACHE: WaveCache = WaveCache::new();
    CACHE.sample(frequency, position, |frequency, position| {
        2.0 / PI
            * (1..)
                .map(|x| x as f32)
                .take_while(|x| x * frequency < 20000.0)
                .map(|n| (2.0 * PI * n * position * frequency).sin() / n)
                .sum::<f32>()
    })
}

pub fn sine(f: f32, t: f32) -> f32 {
    (2.0 * PI * f * t).sin()
}

static NOISE_RNG: OnceCell<Mutex<RNG>> = OnceCell::new();

pub fn noise(_: f32, _: f32) -> f32 {
    let rng = NOISE_RNG.get_or_init(|| Mutex::new(RNG(12345)));
    rng.lock().unwrap().next() as f32 / std::u64::MAX as f32 * 2.0 - 1.0
}
