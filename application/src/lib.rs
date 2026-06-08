#![no_std]

pub struct SmaBuffer<const N: usize> {
    next_element: usize,
    is_initialized: bool,
    inner_buffer: [i32; N],
}

impl<const N: usize> SmaBuffer<N> {
    pub fn new() -> Self {
        Self {
            next_element: 0,
            is_initialized: false,
            inner_buffer: [0; N],
        }
    }

    pub fn push(&mut self, value: i32) {
        if !self.is_initialized {
            self.inner_buffer = [value; N];
            self.is_initialized = true;
        } else {
            self.inner_buffer[self.next_element] = value;
        }

        self.next_element = (self.next_element + 1) % N;
    }

    pub fn average(&self) -> i32 {
        self.inner_buffer.iter().sum::<i32>() / N as i32
    }
    pub fn capacity(&self) -> usize {
        N
    }
}

#[cfg(test)]
mod tests {

    extern crate std;
    use crate::*;
    use std::prelude::v1::*;
    #[test]
    fn average_of_zero_samples() {
        let buffer: SmaBuffer<8> = SmaBuffer::new();
        assert_eq!(buffer.average(), 0);
    }

    #[test]
    fn average_of_single_sample() {
        let mut buffer: SmaBuffer<8> = SmaBuffer::new();
        buffer.push(42);
        assert_eq!(buffer.average(), 42);
    }
    #[test]
    fn average_of_multiple_samples() {
        let mut buffer: SmaBuffer<8> = SmaBuffer::new();
        for i in 0..buffer.capacity() {
            buffer.push(i as i32);
        }
        assert_eq!(buffer.average(), (buffer.capacity() - 1) as i32 / 2);
    }
}
