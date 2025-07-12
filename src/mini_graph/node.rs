use crate::mini_graph::buffer::Frame;
use crate::mini_graph::bang::Bang;

pub trait Node<const N: usize, const C: usize> {
    fn process(&mut self, inputs: &[Frame<N, C>], output: &mut Frame<N, C>){}
    fn handle_bang(&mut self, inputs: &[Bang], output: &mut Bang) { }
}

pub type BoxedNode<const N: usize, const C: usize> = Box<dyn Node<N, C> + Send> ;