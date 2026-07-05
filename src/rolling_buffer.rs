/// Fixed-size rolling buffer that keeps the newest `N` values.
///
/// The buffer does not allocate and works in `no_std` builds. A capacity of
/// `N = 0` is allowed and behaves as a documented no-op buffer: pushed values
/// are dropped, length remains zero, and iteration is always empty.
#[derive(Clone, Debug, PartialEq)]
pub struct RollingBuffer<T, const N: usize> {
    slots: [Option<T>; N],
    start: usize,
    len: usize,
}

impl<T, const N: usize> RollingBuffer<T, N> {
    /// Creates an empty rolling buffer.
    pub fn new() -> Self {
        Self {
            slots: core::array::from_fn(|_| None),
            start: 0,
            len: 0,
        }
    }

    /// Pushes a value, evicting the oldest value when the buffer is full.
    pub fn push(&mut self, value: T) {
        if N == 0 {
            return;
        }

        if self.len < N {
            let index = (self.start + self.len) % N;
            self.slots[index] = Some(value);
            self.len += 1;
        } else {
            self.slots[self.start] = Some(value);
            self.start = (self.start + 1) % N;
        }
    }

    /// Returns the number of values currently stored.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns true when the buffer has no stored values.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the fixed capacity.
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Iterates values from oldest to newest.
    pub fn iter(&self) -> Iter<'_, T, N> {
        Iter {
            buffer: self,
            offset: 0,
        }
    }
}

impl<const N: usize> RollingBuffer<f64, N> {
    /// Returns the arithmetic mean of stored values, or `None` when empty.
    pub fn mean(&self) -> Option<f64> {
        if self.is_empty() {
            return None;
        }

        let sum: f64 = self.iter().copied().sum();
        Some(sum / self.len as f64)
    }
}

impl<T, const N: usize> Default for RollingBuffer<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Iterator over [`RollingBuffer`] values from oldest to newest.
#[derive(Clone, Debug)]
pub struct Iter<'a, T, const N: usize> {
    buffer: &'a RollingBuffer<T, N>,
    offset: usize,
}

impl<'a, T, const N: usize> Iterator for Iter<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.buffer.len {
            return None;
        }

        let index = (self.buffer.start + self.offset) % N;
        self.offset += 1;

        self.buffer.slots[index].as_ref()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.buffer.len.saturating_sub(self.offset);
        (remaining, Some(remaining))
    }
}

impl<T, const N: usize> ExactSizeIterator for Iter<'_, T, N> {}

#[cfg(test)]
mod tests {
    use super::*;

    fn values<const N: usize>(buffer: &RollingBuffer<i32, N>) -> std::vec::Vec<i32> {
        buffer.iter().copied().collect()
    }

    #[test]
    fn empty_buffer_does_not_panic() {
        let buffer = RollingBuffer::<f64, 3>::new();

        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
        assert_eq!(buffer.capacity(), 3);
        assert_eq!(buffer.iter().next(), None);
        assert_eq!(buffer.mean(), None);
    }

    #[test]
    fn single_element_mean_returns_that_element() {
        let mut buffer = RollingBuffer::<f64, 3>::new();

        buffer.push(7.5);

        assert_eq!(buffer.mean(), Some(7.5));
    }

    #[test]
    fn iterates_from_oldest_to_newest() {
        let mut buffer = RollingBuffer::<i32, 3>::new();

        buffer.push(1);
        buffer.push(2);
        buffer.push(3);

        assert_eq!(values(&buffer), [1, 2, 3]);
    }

    #[test]
    fn overflow_evicts_oldest_values() {
        let mut buffer = RollingBuffer::<i32, 3>::new();

        buffer.push(1);
        buffer.push(2);
        buffer.push(3);
        buffer.push(4);

        assert_eq!(buffer.len(), 3);
        assert_eq!(values(&buffer), [2, 3, 4]);
    }

    #[test]
    fn wrap_around_keeps_order() {
        let mut buffer = RollingBuffer::<i32, 3>::new();

        buffer.push(1);
        buffer.push(2);
        buffer.push(3);
        buffer.push(4);
        buffer.push(5);
        buffer.push(6);

        assert_eq!(values(&buffer), [4, 5, 6]);
    }

    #[test]
    fn zero_capacity_buffer_is_documented_noop() {
        let mut buffer = RollingBuffer::<f64, 0>::new();

        buffer.push(1.0);
        buffer.push(2.0);

        assert_eq!(buffer.capacity(), 0);
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
        assert_eq!(buffer.iter().next(), None);
        assert_eq!(buffer.mean(), None);
    }
}
