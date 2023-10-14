//! This crate contains an iterator which will allow you to fully peek any number of elements.
#![forbid(unsafe_code)]

pub fn fully_peek(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = fully_peek(2, 2);
        assert_eq!(result, 4);
    }
}
