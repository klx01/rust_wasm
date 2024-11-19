use std::collections::VecDeque;

#[derive(Debug)]
pub struct RingBuffer<T> {
    inner: VecDeque<T>,
}
impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: VecDeque::with_capacity(capacity),
        }
    }
    pub fn push(&mut self, value: T) {
        if self.inner.len() >= self.inner.capacity() {
            self.inner.pop_front();
        }
        self.inner.push_back(value);
    }
    pub fn as_slices(&self) -> (&[T], &[T]) {
        self.inner.as_slices()
    }
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }
    pub fn truncate(&mut self) {
        self.inner.truncate(0);
    }
}
