//! This crate contains an iterator which allows you to fully peek forward any number of elements.
use std::collections::VecDeque;
use std::iter::FusedIterator;

/// A trait for an interator which allows you to fully peek forward any number of elements.
#[derive(Clone, Debug)]
pub struct FullyPeekableIterator<I: Iterator> {
    iter: I,
    queue: VecDeque<I::Item>,
}

/// Create a new fully-peekable iterator from an existing iterator.
impl<I: Iterator> FullyPeekableIterator<I> {
    fn new(iter: I) -> FullyPeekableIterator<I> {
        FullyPeekableIterator {
            iter,
            queue: VecDeque::new(),
        }
    }
}

/// Implementation of the typical iterator methods on the fully-peekable iterator.
impl<I: Iterator> Iterator for FullyPeekableIterator<I> {
    type Item = I::Item;

    /// Returns the next value which may advance the iterator.
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.queue.pop_front().or_else(|| self.iter.next())
    }

    /// Returns the bounds on the remaining length of the iterator.
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let peek_len = self.queue.len();
        let (lo, hi) = self.iter.size_hint();
        let lo = lo.saturating_add(peek_len);
        let hi = match hi {
            Some(x) => x.checked_add(peek_len),
            None => None,
        };
        (lo, hi)
    }
}

// TODO: Implement `DoubleEndedIterator` for `FullyPeekableIterator`?

impl<I: ExactSizeIterator> ExactSizeIterator for FullyPeekableIterator<I> {}

impl<I: FusedIterator> FusedIterator for FullyPeekableIterator<I> {}

impl<I: Iterator> FullyPeekableIterator<I> {
    /// Test if the iterator has another element to yield. May advance the underlying iterator.
    #[inline]
    pub fn has_next(&mut self) -> bool {
        self.peek().is_some()
    }

    /// Peek forward to an arbitrary element without advancing the iterator.
    #[inline]
    pub fn lift(&mut self, index: usize) -> Option<&I::Item> {
        while self.queue.len() < index + 1 {
            match self.iter.next() {
                Some(item) => self.queue.push_back(item),
                None => break,
            }
        }
        self.queue.get(index)
    }

    /// Peek forward to a range of arbitrary elements without advancing the iterator.
    #[inline]
    pub fn lift_many(&mut self, start: usize, end: usize) -> Vec<Option<&I::Item>> {
        self.lift(end.max(1) - 1); // Ensure we've filled the queue with as many items as necessary.
        let mut result = Vec::with_capacity(end - start);
        for index in start..end {
            result.push(self.queue.get(index));
        }
        result
    }

    /// Peek forward to an arbitrary mutable element without advancing the iterator.
    #[inline]
    pub fn lift_mut(&mut self, index: usize) -> Option<&mut I::Item> {
        while self.queue.len() <= index + 1 {
            match self.iter.next() {
                Some(item) => self.queue.push_back(item),
                None => break,
            }
        }
        self.queue.get_mut(index)
    }

    /// Peek forward to the next element without advancing the iterator.
    #[inline]
    pub fn peek(&mut self) -> Option<&I::Item> {
        self.lift(0)
    }

    /// Peek forward to a set number of arbitrary elements without advancing the iterator.
    #[inline]
    pub fn peek_many(&mut self, n: usize) -> Vec<Option<&I::Item>> {
        self.lift_many(0, n)
    }

    /// Peek forward to the next element marking it as mutable without advancing the iterator.
    #[inline]
    pub fn peek_mut(&mut self) -> Option<&mut I::Item> {
        self.lift_mut(0)
    }

    /// Consume and return the next value of this iterator if a condition is true.
    #[inline]
    pub fn next_if(&mut self, func: impl FnOnce(&I::Item) -> bool) -> Option<I::Item> {
        match self.next() {
            Some(matched) if func(&matched) => Some(matched),
            Some(other) => {
                self.queue.push_front(other);
                None
            }
            None => None,
        }
    }

    /// Consume and return the next item if it is equal to `expected`.
    #[inline]
    pub fn next_if_eq<T>(&mut self, expected: &T) -> Option<I::Item>
    where
        T: ?Sized,
        I::Item: PartialEq<T>,
    {
        self.next_if(|next| next == expected)
    }
}

/// A trait for an iterator which allows you to fully peek forward any number of elements.
pub trait IntoFullyPeekableIterator<I>
where
    I: Iterator,
{
    /// Return a fully peekable iterator.
    fn fully_peekable(self) -> FullyPeekableIterator<I>;
}

/// Add a fully-peekable iterator implementation to any iterator implicitly.
impl<I, T> IntoFullyPeekableIterator<I> for I
where
    I: Iterator<Item = T>,
{
    /// Return a fully peekable iterator.
    fn fully_peekable(self) -> FullyPeekableIterator<I> {
        FullyPeekableIterator::new(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::{FullyPeekableIterator, IntoFullyPeekableIterator};

    #[test]
    fn the_class_returns_elements_like_an_iterator_when_using_next() {
        let iter = vec![1, 2].into_iter();
        let mut peekable = FullyPeekableIterator::new(iter);
        assert_eq!(peekable.next(), Some(1));
        assert_eq!(peekable.next(), Some(2));
        assert_eq!(peekable.next(), None);
    }

    #[test]
    fn it_uses_has_next_to_determine_if_there_are_more_elements() {
        let iter = vec![1, 2].into_iter();
        let mut peekable = FullyPeekableIterator::new(iter);
        assert_eq!(peekable.has_next(), true);
        assert_eq!(peekable.next(), Some(1));
        assert_eq!(peekable.has_next(), true);
        assert_eq!(peekable.next(), Some(2));
        assert_eq!(peekable.has_next(), false);
        assert_eq!(peekable.next(), None);
        assert_eq!(peekable.has_next(), false);
    }

    #[test]
    fn it_can_estimate_its_size_using_size_hint() {
        let iter = vec![1, 2].into_iter();
        let mut peekable = FullyPeekableIterator::new(iter);
        assert_eq!(peekable.size_hint(), (2, Some(2)));
        assert_eq!(peekable.next(), Some(1));
        assert_eq!(peekable.size_hint(), (1, Some(1)));
        assert_eq!(peekable.next(), Some(2));
        assert_eq!(peekable.size_hint(), (0, Some(0)));
        assert_eq!(peekable.next(), None);
        assert_eq!(peekable.size_hint(), (0, Some(0)));
    }

    #[test]
    fn it_can_estimate_a_size_even_if_the_iterator_has_no_high() {
        struct TestIterator<I: Iterator> {
            iter: I,
        }

        impl<I: Iterator> TestIterator<I> {
            fn new(iter: I) -> TestIterator<I> {
                TestIterator { iter }
            }
        }

        impl<I: Iterator> Iterator for TestIterator<I> {
            type Item = I::Item;

            fn next(&mut self) -> Option<Self::Item> {
                self.iter.next()
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                (0, None)
            }
        }

        let iter = vec![1, 2].into_iter();
        let mut peekable = FullyPeekableIterator::new(TestIterator::new(iter));
        assert_eq!(peekable.size_hint(), (0, None));
        assert_eq!(peekable.peek(), Some(&1));
        assert_eq!(peekable.lift(0), Some(&1));
        assert_eq!(peekable.size_hint(), (1, None));
        assert_eq!(peekable.peek_many(3), vec!(Some(&1), Some(&2), None));
        assert_eq!(peekable.size_hint(), (2, None));
        assert_eq!(peekable.next(), Some(1));
        assert_eq!(peekable.size_hint(), (1, None));
        assert_eq!(peekable.next(), Some(2));
        assert_eq!(peekable.size_hint(), (0, None));
    }

    #[test]
    fn it_can_lift_elements_without_advancing() {
        let iter = vec![1, 2].into_iter();
        let mut peekable = FullyPeekableIterator::new(iter);
        assert_eq!(peekable.lift(0), Some(&1));
        assert_eq!(peekable.lift(1), Some(&2));
        assert_eq!(peekable.lift(2), None);
        assert_eq!(peekable.next(), Some(1));
        assert_eq!(peekable.next(), Some(2));
        assert_eq!(peekable.next(), None);
        assert_eq!(peekable.lift(10), None);
    }

    #[test]
    fn it_can_lift_many_elements_without_advancing() {
        let iter = vec![1, 2].into_iter();
        let mut peekable = FullyPeekableIterator::new(iter);
        assert_eq!(peekable.lift_many(0, 1), vec!(Some(&1)));
        assert_eq!(peekable.lift_many(0, 2), vec!(Some(&1), Some(&2)));
        assert_eq!(peekable.lift_many(1, 3), vec!(Some(&2), None));
        assert_eq!(peekable.lift_many(5, 7), vec!(None, None));
        assert_eq!(peekable.next(), Some(1));
        assert_eq!(peekable.next(), Some(2));
        assert_eq!(peekable.next(), None);
    }

    #[test]
    fn it_can_peek_at_the_next_element_without_advancing() {
        let iter = vec![1, 2].into_iter();
        let mut peekable = iter.fully_peekable();
        assert_eq!(peekable.peek(), Some(&1));
        assert_eq!(peekable.next(), Some(1));
        assert_eq!(peekable.peek(), Some(&2));
        assert_eq!(peekable.next(), Some(2));
        assert_eq!(peekable.peek(), None);
        assert_eq!(peekable.next(), None);
    }

    #[test]
    fn it_can_peek_many_elements_without_advancing() {
        let iter = vec![1, 2].into_iter();
        let mut peekable = iter.fully_peekable();
        assert_eq!(peekable.peek_many(0), vec!());
        assert_eq!(peekable.peek_many(1), vec!(Some(&1)));
        assert_eq!(peekable.peek_many(2), vec!(Some(&1), Some(&2)));
        assert_eq!(peekable.peek_many(3), vec!(Some(&1), Some(&2), None));
        assert_eq!(peekable.next(), Some(1));
        assert_eq!(peekable.peek_many(1), vec!(Some(&2)));
        assert_eq!(peekable.peek_many(2), vec!(Some(&2), None));
        assert_eq!(peekable.next(), Some(2));
        assert_eq!(peekable.peek_many(1), vec!(None));
        assert_eq!(peekable.peek_many(2), vec!(None, None));
        assert_eq!(peekable.peek_many(0), vec!());
    }

    #[test]
    fn it_can_lift_elements_without_advancing_mut() {
        let iter = vec![1, 2].into_iter();
        let mut peekable = FullyPeekableIterator::new(iter);
        assert_eq!(peekable.lift_mut(0), Some(&mut 1));
        assert_eq!(peekable.lift_mut(1), Some(&mut 2));
        assert_eq!(peekable.lift_mut(2), None);
        assert_eq!(peekable.next(), Some(1));
        assert_eq!(peekable.next(), Some(2));
        assert_eq!(peekable.next(), None);
    }

    #[test]
    fn it_can_peek_at_the_next_element_without_advancing_mut() {
        let iter = vec![1, 2].into_iter();
        let mut peekable = iter.fully_peekable();
        assert_eq!(peekable.peek_mut(), Some(&mut 1));
        assert_eq!(peekable.next(), Some(1));
        assert_eq!(peekable.peek_mut(), Some(&mut 2));
        assert_eq!(peekable.next(), Some(2));
        assert_eq!(peekable.peek_mut(), None);
        assert_eq!(peekable.next(), None);
    }

    #[test]
    fn it_can_return_the_next_element_if_a_predicate_is_true() {
        let iter = vec![1, 2].into_iter();
        let mut peekable = iter.fully_peekable();
        assert_eq!(peekable.next_if(|next| next == &0), None);
        assert_eq!(peekable.next_if(|next| next == &1), Some(1));
        assert_eq!(peekable.next_if(|next| next == &1), None);
        assert_eq!(peekable.next_if(|next| next == &2), Some(2));
        assert_eq!(peekable.has_next(), false);
        assert_eq!(peekable.next_if(|_| true), None);
    }

    #[test]
    fn it_can_return_the_next_element_if_it_is_equal_to_a_supplied_value() {
        let iter = vec![1, 2].into_iter();
        let mut peekable = iter.fully_peekable();
        assert_eq!(peekable.next_if_eq(&0), None);
        assert_eq!(peekable.next_if_eq(&1), Some(1));
        assert_eq!(peekable.next_if_eq(&1), None);
        assert_eq!(peekable.next_if_eq(&2), Some(2));
        assert_eq!(peekable.has_next(), false);
    }
}
