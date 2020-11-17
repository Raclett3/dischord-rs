use crate::parse::{Instruction, ParseError, ParseResult, RollbackableTokenStream};

fn hex_to_num(hex: u8) -> Option<usize> {
    if b'0' <= hex && hex <= b'9' {
        Some((hex - b'0') as usize)
    } else if b'a' <= hex && hex <= b'f' {
        Some((hex - b'a') as usize + 10)
    } else {
        None
    }
}

pub fn tone(stream: &mut RollbackableTokenStream) -> ParseResult {
    if stream.expect_character('@').is_err() {
        return Ok(None);
    }

    if let Ok((_, number)) = stream.take_number() {
        return Ok(Some(Instruction::Tone(number)));
    }

    let (inst_at, inst) = stream.take_character()?;

    match inst {
        'd' => {
            let token_at = stream.cursor();
            let params = stream.comma_separated_numbers();
            if params.len() != 2 {
                return Err(ParseError::WrongParamsNumber(token_at, 2, params.len()));
            }
            Ok(Some(Instruction::Detune(
                params[0],
                params[1] as f32 / 10000.0,
            )))
        }
        'e' => {
            let token_at = stream.cursor();
            let params = stream.comma_separated_numbers();
            if params.len() != 4 {
                return Err(ParseError::WrongParamsNumber(token_at, 4, params.len()));
            }
            let params: Vec<_> = params.iter().map(|&x| x as f32 / 100.0).collect();
            let envelope = Instruction::Envelope(params[0], params[1], params[2], params[3]);
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
            Ok(Some(Instruction::DefinePCMTone(pcm)))
        }
        'p' => Ok(Some(Instruction::PCMTone(stream.take_number()?.1))),
        'g' => {
            let (_, gate) = stream.take_number()?;
            Ok(Some(Instruction::Gate(gate as f32 / 1000.0)))
        }
        't' => {
            let (_, tune) = stream.take_number()?;
            Ok(Some(Instruction::Tune(tune as f32 / 1000.0)))
        }
        _ => Err(ParseError::unexpected_char(inst_at, inst)),
    }
}
