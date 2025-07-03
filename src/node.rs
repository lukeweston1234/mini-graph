use crate::buffer::Frame;

pub trait Node<const N: usize, const C: usize> {
    fn process(&mut self, inputs: &[Frame<N, C>], output: &mut Frame<N, C>){}
}

pub type BoxedNode<const N: usize, const C: usize> = Box<dyn Node<N, C> + Send> ;