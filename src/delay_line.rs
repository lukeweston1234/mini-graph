use std::collections::VecDeque;

use crate::node::Node;
use crate::buffer::{Frame};

/// A multichannel delay line
pub struct DelayLine<const FRAME_SIZE: usize, const CHANNELS: usize> {
    ringbuf: VecDeque<f32>
}

impl<const N: usize, const C: usize> DelayLine<N, C>{
    pub fn new(sample_size: usize) -> Self {
        Self {
            ringbuf: VecDeque::with_capacity(sample_size)
        }
    }
    #[inline(always)]
    fn tick(&mut self, input: f32) -> f32 {
        let sample = self.ringbuf.pop_front();
        self.ringbuf.push_back(input);
        sample.unwrap_or(0.0)
    }
}
impl <const N: usize, const C: usize> Node<N,C> for DelayLine<N,C> {
    #[inline(always)]
    fn process(&mut self, inputs: &[Frame<N, C>], output: &mut Frame<N, C>) {
        let input = inputs[0];
        for n in 0..N {
            for c in 0..C {
                let sample = self.tick(input[c][n]);
                output[c][n] = sample
            }
        }
    }
}