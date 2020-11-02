mod tokenize;

pub fn hello() {
    println!("Hello, world");
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn tokenize_test() {
        use tokenize::*;
        use Token::*;
        assert!(tokenize("Do some 焼き松茸").is_err());
        assert!(tokenize("9999999999999999999999999999999999999999999999999").is_err());
        assert_eq!(tokenize("c256e16g4<ceg4"), Ok(vec![
            Character(b'c'),
            Number(256),
            Character(b'e'),
            Number(16),
            Character(b'g'),
            Number(4),
            Character(b'<'),
            Character(b'c'),
            Character(b'e'),
            Character(b'g'),
            Number(4),
        ]));
    }
}
