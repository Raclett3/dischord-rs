use composer::*;

fn assert_float_eq(a: f64, b: f64) {
    if (a - b).abs() > 1e-8 {
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
        note_length_to_float(vec![DefaultLength, Dot, Dot, Length(2), Dot], 4),
        1. / 4. + 1. / 8. + 1. / 16. + 1. / 2. + 1. / 4.,
    );
}

fn pulse(frequency: f64, position: f64) -> f64 {
    if frequency * position % 1.0 >= 0.5 {
        1.0
    } else {
        -1.0
    }
}

#[test]
fn test_note() {
    use generate::note::Note;

    let note = Note::new(10.0, |_, _| 1.0, 1.0, 0.0, 1.0, 2.0);
    assert!(note.is_waiting(0.0));
    assert!(note.is_ringing(1.0));
    assert!(note.is_over(2.0));
    assert_float_eq(note.get_sample(0.5), 0.0);
    assert_float_eq(note.get_sample(2.0), 0.0);
    for i in 0..100 {
        let position = i as f64 / 100.0;
        assert_float_eq(note.get_sample(position + 1.0), 1.0 - position);
    }

    let note = Note::new(10.0, pulse, 1.0, 1.0, 0.0, 1.0);
    for i in 0..10 {
        let position = i as f64 / 10.0;
        assert_float_eq(note.get_sample(position + 0.025), -1.0);
        assert_float_eq(note.get_sample(position + 0.075), 1.0);
    }
}
