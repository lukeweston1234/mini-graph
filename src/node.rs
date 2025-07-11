use crate::buffer::Frame;

#[derive(Clone, Copy, Debug)]
pub enum Bang {
    Bang,
    BangF32(f32),
    BangU32(u32),
    BangBool(bool),
    BangUSize(usize),
    SetParamU32(usize, u32),
    SetParamF32(usize, f32),
    SetParamBool(usize, bool),
    Empty,
}

pub trait Node<const N: usize, const C: usize> {
    fn process(&mut self, inputs: &[Frame<N, C>], output: &mut Frame<N, C>){}
    fn handle_bang(&mut self, inputs: &[Bang], output: &mut Bang) { }
}

pub type BoxedNode<const N: usize, const C: usize> = Box<dyn Node<N, C> + Send> ;