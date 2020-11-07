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
