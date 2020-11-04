pub mod tokenize;

pub fn hello() {
    println!("Hello, world");
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn tokenize_test() {
        use tokenize::*;
        use TokenKind::*;
        assert!(tokenize("Do some 焼き松茸").is_err());
        assert!(tokenize("9999999999999999999999999999999999999999999999999").is_err());
        assert_eq!(tokenize("c256e16g4<ceg4"), Ok(vec![
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
        ]));
    }
}
