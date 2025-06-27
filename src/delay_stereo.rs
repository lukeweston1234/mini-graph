use std::collections::VecDeque;
use std::time::Duration;

use crate::node::Node;
use crate::buffer::Buffer;

const MAX_DELAY_TIME: Duration = Duration::from_secs(6);

pub struct DelayStereo<const N: usize> {
    delay_lines: [VecDeque<f32>; 2],
    sample_rate: u32,
}
impl<const N: usize> DelayStereo<N>{

    pub fn new(left_time: Duration, right_time: Duration, sample_rate: u32) -> Self {
        assert!(left_time <= MAX_DELAY_TIME && right_time <= MAX_DELAY_TIME);
        let left_buffer_size = sample_rate as f32 * left_time.as_secs_f32();
        let right_buffer_size = sample_rate as f32 * right_time.as_secs_f32();
        Self {
            delay_lines: [
                VecDeque::with_capacity(left_buffer_size as usize),
                VecDeque::with_capacity(right_buffer_size as usize)],
            sample_rate,
        }
    }
    #[inline]
    fn tick_delay(&mut self) -> (f32, f32) {
        (0.0,0.0)
    }
}
impl<const N: usize> Node<N> for DelayStereo<N>{
    fn process(&mut self, input: &[Buffer<N>], output: &mut [Buffer<N>]) {
        for i in 0..N {
            let sample = self.tick_delay();
            output[0][i] = sample.0;
            output[1][i] = sample.1;
        }
    }
}