pub fn hello() {
    println!("Hello, world");
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        assert_eq!(1 + 2, 3);
    }
}
