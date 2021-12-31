#[macro_export]
macro_rules! split_once {
    ($str:ident, $pat:expr) => {
        (|| {
            let mut split = $str.splitn(2, $pat);
            match (split.next(), split.next()) {
                (Some(i1), Some(i2)) => Some((i1, i2)),
                _ => None,
            }
        })()
    };
    ($str:expr, $pat:expr) => {
        (|| {
            let mut split = $str.splitn(2, $pat);
            match (split.next(), split.next()) {
                (Some(i1), Some(i2)) => Some((i1, i2)),
                _ => None,
            }
        })()
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_split_once() {
        let result = split_once!("kek/lol", "/").expect("two items");
        assert_eq!(result, ("kek", "lol"));

        let result = split_once!("git@github.com:@path", "@").expect("two items");
        assert_eq!(result, ("git", "github.com:@path"));

        assert!(split_once!("github.com", "@").is_none());
    }
}
