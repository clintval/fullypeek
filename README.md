# fullypeek

[![Build Status](https://github.com/clintval/fullypeek/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/clintval/fullypeek/actions/workflows/rust.yml)
[![Coverage Status](https://coveralls.io/repos/github/clintval/fullypeek/badge.svg?branch=main)](https://coveralls.io/github/clintval/fullypeek?branch=main)
[![Language](https://img.shields.io/badge/language-rust-DEA584.svg)](https://www.rust-lang.org/)

Peek forward in an iterator as far as you'd like, memory allowing!

```console
cargo add fullypeek
```

![El Chorro, Spain](.github/img/cover.jpg)

```rust
let mut peekable = vec![1, 2, 3].into_iter().fully_peekable();

assert_eq!(peekable.peek(), Some(&1));
assert_eq!(peekable.peek_many(2), vec!(Some(&1), Some(&2)));

peekable.next();

assert_eq!(peekable.peek(), Some(&2));
assert_eq!(peekable.peek_many(2), vec!(Some(&2), Some(&3)));

peekable.next();

assert_eq!(peekable.peek(), Some(&3));
assert_eq!(peekable.peek_many(2), vec!(Some(&3), None));
```
