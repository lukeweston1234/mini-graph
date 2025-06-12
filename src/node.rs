use crate::buffer::Buffer;

pub trait Node<const N: usize> {
    fn process(&mut self, input: &[Buffer<N>], output: &mut [Buffer<N>]);
}
