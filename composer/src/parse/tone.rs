use crate::parse::{Instruction, ParseError, ParseResult, RollbackableTokenStream, ToneModifier};

fn hex_to_num(hex: u8) -> Option<usize> {
    if b'0' <= hex && hex <= b'9' {
        Some((hex - b'0') as usize)
    } else if b'a' <= hex && hex <= b'f' {
        Some((hex - b'a') as usize + 10)
    } else if b'A' <= hex && hex <= b'F' {
        Some((hex - b'A') as usize + 10)
    } else {
        None
    }
}

#[derive(PartialEq, Debug)]
pub enum Effect {
    Delay { delay: f32, feedback: f32 },
    LowPassFilter { cut_off: f32 },
    HighPassFilter { cut_off: f32 },
}

fn effects(stream: &mut RollbackableTokenStream) -> ParseResult {
    let (effect_at, effect) = stream.take_character()?;

    match effect {
        'd' => {
            let params: Vec<_> = stream
                .comma_separated_n_numbers(2)?
                .into_iter()
                .map(|x| x as f32 / 1000.0)
                .collect();

            let delay = params[0];
            let feedback = params[1];
            Ok(Some(Instruction::ToneModifier(ToneModifier::Effect(
                Effect::Delay { delay, feedback },
            ))))
        }
        'l' => {
            let params = stream.comma_separated_n_numbers(1)?;

            let cut_off = params[0] as f32;
            Ok(Some(Instruction::ToneModifier(ToneModifier::Effect(
                Effect::LowPassFilter { cut_off },
            ))))
        }
        'h' => {
            let params = stream.comma_separated_n_numbers(1)?;

            let cut_off = params[0] as f32;
            Ok(Some(Instruction::ToneModifier(ToneModifier::Effect(
                Effect::HighPassFilter { cut_off },
            ))))
        }
        _ => Err(ParseError::unexpected_char(effect_at, effect)),
    }
}

static BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn base64_to_bits(ch: u8) -> usize {
    BASE64_CHARS.iter().position(|&x| x == ch).unwrap_or(0)
}

pub fn tone(stream: &mut RollbackableTokenStream) -> ParseResult {
    if stream.expect_character('@').is_err() {
        return Ok(None);
    }

    if let Ok((_, number)) = stream.take_number() {
        return Ok(Some(Instruction::ToneModifier(ToneModifier::Tone(number))));
    }

    let (inst_at, inst) = stream.take_character()?;

    match inst {
        'd' => {
            let params = stream.comma_separated_n_numbers(2)?;
            Ok(Some(Instruction::ToneModifier(ToneModifier::Detune(
                params[0],
                params[1] as f32 / 10000.0,
            ))))
        }
        'e' => {
            let params = stream.comma_separated_n_numbers(4)?;
            let params: Vec<_> = params.iter().map(|&x| x as f32 / 100.0).collect();
            let envelope = Instruction::ToneModifier(ToneModifier::Envelope(
                params[0], params[1], params[2], params[3],
            ));
            Ok(Some(envelope))
        }
        'h' => {
            let (_, string) = stream.take_brace_string()?;
            let pcm = string
                .bytes()
                .map(|byte| {
                    hex_to_num(byte)
                        .map(|x| (x as f32 - 8.0) / 8.0)
                        .unwrap_or(0.0)
                })
                .collect();
            Ok(Some(Instruction::ToneModifier(
                ToneModifier::DefinePCMTone(pcm),
            )))
        }
        'n' => {
            let (_, string) = stream.take_brace_string()?;
            let pcm = string
                .bytes()
                .flat_map(|byte| {
                    let bits = base64_to_bits(byte);
                    (0..6)
                        .rev()
                        .map(move |i| if bits >> i & 1 == 0 { -1 } else { 1 })
                })
                .scan(0i8, |acc, x| {
                    let (sum, _) = acc.overflowing_add(x);
                    *acc = sum;
                    Some(*acc)
                })
                .map(|x| x as f32 / 128.0)
                .collect();
            Ok(Some(Instruction::ToneModifier(
                ToneModifier::DefinePCMTone(pcm),
            )))
        }
        'p' => Ok(Some(Instruction::ToneModifier(ToneModifier::PCMTone(
            stream.take_number()?.1,
        )))),
        'g' => {
            let (_, gate) = stream.take_number()?;
            Ok(Some(Instruction::ToneModifier(ToneModifier::Gate(
                gate as f32 / 1000.0,
            ))))
        }
        'v' => {
            let (_, volume) = stream.take_number()?;
            Ok(Some(Instruction::ToneModifier(ToneModifier::Volume(
                volume as f32 / 100.0,
            ))))
        }
        't' => {
            let (_, tune) = stream.take_number()?;
            Ok(Some(Instruction::ToneModifier(ToneModifier::Tune(
                tune as f32 / 1000.0,
            ))))
        }
        'f' => effects(stream),
        _ => Err(ParseError::unexpected_char(inst_at, inst)),
    }
}

pub fn synthesize(stream: &mut RollbackableTokenStream) -> ParseResult {
    if stream.expect_character('@').is_err() || stream.expect_character('(').is_err() {
        return Ok(None);
    }

    let mut tones = vec![vec![]];

    while stream.expect_character(')').is_err() {
        if stream.expect_character(',').is_ok() || stream.expect_character('|').is_ok() {
            tones.push(vec![]);
            continue;
        }

        if let Some(Instruction::ToneModifier(modifier)) = tone(stream)? {
            tones.last_mut().unwrap().push(modifier);
        } else {
            if let Some(token) = stream.next() {
                return Err(ParseError::UnexpectedToken(token.clone()));
            } else {
                return Err(ParseError::UnexpectedEOF);
            }
        }
    }

    Ok(Some(Instruction::Synthesize(tones)))
}
