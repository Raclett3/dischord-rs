extern crate cpal;

use composer::generate::Generator;
use composer::parse::{parse, Instruction};
use composer::tokenize::tokenize;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::io::{stdin, Read, Result};
use std::sync::{
    atomic::{AtomicBool, Ordering::SeqCst},
    Arc,
};

fn main() -> Result<()> {
    let mut buf = String::new();
    stdin().lock().read_to_string(&mut buf)?;
    let tokens = match tokenize(&buf) {
        Ok(tokens) => tokens,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    };
    let parsed = match parse(&tokens) {
        Ok(parsed) => parsed,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    };

    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .expect("failed to find a default output device");
    let config = device.default_output_config().unwrap();

    match config.sample_format() {
        cpal::SampleFormat::F32 => play::<f32>(&device, &config.into(), &parsed).unwrap(),
        cpal::SampleFormat::I16 => play::<i16>(&device, &config.into(), &parsed).unwrap(),
        cpal::SampleFormat::U16 => play::<u16>(&device, &config.into(), &parsed).unwrap(),
    }
    Ok(())
}

fn play<T: cpal::Sample>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    tracks: &[Vec<Instruction>],
) -> Option<()> {
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    println!("generating...");
    let mut generator = Generator::new(sample_rate, tracks);
    println!("generated! length: {:.2}s", generator.track_length());
    let is_over = Arc::new(AtomicBool::new(false));
    let is_over_cloned = is_over.clone();

    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [T], _| write_samples(data, channels, &mut generator, &is_over_cloned),
            |err| eprintln!("an error occurred on stream: {}", err),
        )
        .ok()?;
    stream.play().ok()?;

    println!("playing...");
    while !is_over.load(SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    Some(())
}

fn write_samples<T: cpal::Sample>(
    data: &mut [T],
    channels: usize,
    generator: &mut Generator,
    is_over: &Arc<AtomicBool>,
) {
    for frame in data.chunks_mut(channels) {
        let sample = generator.next().unwrap_or_else(|| {
            is_over.store(true, SeqCst);
            0.0
        });
        let value: T = cpal::Sample::from(&sample);
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
