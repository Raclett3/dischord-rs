use composer::*;

fn assert_float_eq(a: f32, b: f32) {
    if (a - b).abs() > 1e-5 {
        panic!(
            "assertion failed: `(left == right)`\n  left: `{}`,\n right: `{}`",
            a, b
        );
    }
}

#[test]
fn test_note_length_to_float() {
    use generate::note_length_to_float;
    use parse::NoteLength::*;

    assert_float_eq(
        note_length_to_float(&[DefaultLength, Dot, Dot, Length(2), Dot], 1. / 4.),
        1. / 4. + 1. / 8. + 1. / 16. + 1. / 2. + 1. / 4.,
    );
}

fn pulse(frequency: f32, position: f32) -> f32 {
    if frequency * position % 1.0 >= 0.5 {
        1.0
    } else {
        -1.0
    }
}

#[test]
fn test_note() {
    use generate::note::Note;
    use generate::Tone::FnTone;

    let note = Note::new(10.0, FnTone(|_, _| 1.0), 1.0, 0.0, 0.0, 1.0, 2.0);
    assert!(note.is_waiting(0.0));
    assert!(note.is_ringing(1.0));
    assert!(note.is_over(2.0));
    assert_float_eq(note.get_sample(0.5), 0.0);
    assert_float_eq(note.get_sample(2.0), 0.0);
    for i in 0..100 {
        let position = i as f32 / 100.0;
        assert_float_eq(note.get_sample(position + 1.0), 1.0 - position);
    }

    let note = Note::new(10.0, FnTone(pulse), 1.0, 1.0, 0.0, 0.0, 1.0);
    for i in 0..10 {
        let position = i as f32 / 10.0;
        assert_float_eq(note.get_sample(position + 0.025), -1.0);
        assert_float_eq(note.get_sample(position + 0.075), 1.0);
    }

    let note = Note::new(10.0, FnTone(pulse), 1.0, 1.0, 0.05, 0.0, 1.0);
    for i in 0..10 {
        let position = i as f32 / 10.0;
        assert_float_eq(note.get_sample(position + 0.025), 1.0);
        assert_float_eq(note.get_sample(position + 0.075), -1.0);
    }
}

#[test]
fn test_note_queue() {
    use generate::note::{Note, NotesQueue};
    use generate::Tone::FnTone;

    let note_a = Note::new(10.0, FnTone(pulse), 0.8, 0.9, 0.0, 3.0, 5.0);
    let note_b = Note::new(20.0, FnTone(pulse), 1.0, 0.9, 0.0, 1.0, 6.0);
    let note_c = Note::new(30.0, FnTone(pulse), 0.9, 1.0, 0.0, 2.0, 4.0);
    let mut queue = NotesQueue::new(vec![note_a.clone(), note_b.clone(), note_c.clone()]);
    assert_eq!(queue.next_before(0.5), None);
    assert_eq!(queue.next_before(1.0), Some(note_b));
    assert_eq!(queue.next_before(1.0), None);
    assert_eq!(queue.next_before(2.0), Some(note_c));
    assert_eq!(queue.next_before(2.0), None);
    assert_eq!(queue.next_before(3.0), Some(note_a));
    assert_eq!(queue.next_before(3.0), None);
    assert_eq!(queue.next_before(10.0), None);
}

#[test]
fn test_tone() {
    use generate::Tone;
    use std::sync::Arc;

    let fn_tone = Tone::FnTone(|a, b| a * b);
    assert_float_eq(fn_tone.sample(10.0, 20.0), 200.0);

    let pcm_tone = Tone::PCMTone(Arc::new(vec![0.0, 1.0, 2.0, 3.0, 4.0]));
    assert_float_eq(pcm_tone.sample(0.2, 1.5), 1.0);
    assert_float_eq(pcm_tone.sample(0.2, 7.5), 2.0);
}
