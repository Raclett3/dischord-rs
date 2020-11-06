use composer::*;
#[test]
fn token_test() {
    use tokenize::TokenKind;

    assert!(TokenKind::Character(b'c').is_character());
    assert!(TokenKind::Number(42).is_number());
}

#[test]
fn tokenize_test() {
    use tokenize::*;
    use TokenKind::*;
    assert!(tokenize("Do some 焼き松茸").is_err());
    assert!(tokenize("9999999999999999999999999999999999999999999999999").is_err());
    assert_eq!(
        tokenize("c256e16g4<CEG4"),
        Ok(vec![
            (1, Character(b'c')),
            (2, Number(256)),
            (5, Character(b'e')),
            (6, Number(16)),
            (8, Character(b'g')),
            (9, Number(4)),
            (10, Character(b'<')),
            (11, Character(b'c')),
            (12, Character(b'e')),
            (13, Character(b'g')),
            (14, Number(4)),
        ])
    );
    assert_eq!(
        tokenize("C e\n\rG"),
        Ok(vec![
            (1, Character(b'c')),
            (3, Character(b'e')),
            (6, Character(b'g')),
        ])
    );
}
