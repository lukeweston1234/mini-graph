use std::collections::VecDeque;

use crate::mini_graph::node::Node;
use crate::mini_graph::buffer::{Frame};


pub struct DelayLine<const N: usize, const C: usize> {
    ringbufs: [VecDeque<f32>; C],
}

impl<const N: usize, const C: usize> DelayLine<N, C>{
    pub fn new(delay_len: usize) -> Self {
        let ringbufs = std::array::from_fn(|_| {
            let mut buf = VecDeque::with_capacity(delay_len);
            buf.extend(std::iter::repeat(0.0).take(delay_len));
            buf
        });
        Self { ringbufs }
    }

    #[inline(always)]
    fn tick(&mut self, chan: usize, input: f32) -> f32 {
        let buf = &mut self.ringbufs[chan];
        let out = buf.pop_front().unwrap();
        buf.push_back(input);
        out
    }
}

impl<const N: usize, const C: usize> Node<N, C> for DelayLine<N, C> {
    #[inline(always)]
    fn process(&mut self, inputs: &[Frame<N, C>], output: &mut Frame<N, C>) {
        let input = inputs[0];
        for n in 0..N {
            for c in 0..C {
                output[c][n] = self.tick(c, input[c][n]);
            }
        }
    }
}
