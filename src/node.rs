use crate::buffer::Buffer;
use crate::oscillator::Oscillator;
use crate::adsr::ADSR;

pub trait Node<const N: usize> {
    fn process(&mut self, input: &[Buffer<N>], output: &mut [Buffer<N>]);
}

/// Fow now, we are using an enum to avoid Box + dyn
/// 
/// This is probably some sort of premature optimization
pub enum PipelineNode<const N: usize> {
    OscillatorNode(Oscillator<N>),
    ADSRNode(ADSR<N>)
}
impl<const N: usize> Node<N> for PipelineNode<N> {
    fn process(&mut self, input: &[Buffer<N>], output: &mut [Buffer<N>]) {
        match self {
            PipelineNode::OscillatorNode(node) => node.process(input, output),
            PipelineNode::ADSRNode(node) => node.process(input, output),
        }
    }
}

