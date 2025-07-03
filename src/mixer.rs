use crate::node::Node;
use crate::buffer::Frame;

#[derive(Default)]
pub struct Mixer<const N: usize> {}

impl<const N: usize, const C: usize> Node<N, C> for Mixer<N> {
    fn process(&mut self, inputs: &[Frame<N, C>], outputs: &mut Frame<N, C>){
        for n in 0..N {
            for c in 0..C {
                let mut sum = 0.0;
                for input in inputs {
                    sum += input[c][n]
                }
                outputs[c][n] = (sum / inputs.len() as f32).clamp(-1.0, 1.0);
            }
        }
    }
}