use std::time::Duration;

use assert_no_alloc::permit_alloc;

use crate::node::{Node, Bang};
use crate::buffer::Frame;

pub struct Clock<const N: usize, const C: usize> {
    sample_rate: f32,
    is_ticking: bool,
    time_ellapsed: Duration,
    clock_rate: Duration
}
impl<const N: usize, const C: usize> Clock <N, C> {
    pub fn new(sample_rate: u32, clock_rate: Duration) -> Self {
        Self {
            sample_rate: sample_rate as f32,
            is_ticking: true,
            clock_rate,
            time_ellapsed: Duration::from_secs(0)
        }
    }
}
impl<const N: usize, const C: usize>  Node<N,C> for Clock<N, C>{
    fn process(&mut self, _: &[Frame<N, C>], _: &mut Frame<N, C>) {
        return
    }
    fn handle_bang(&mut self, inputs: &[Bang], output: &mut Bang) {
        permit_alloc(|| {
            if let Some(ref bang) = inputs.get(0){
                match bang {
                    Bang::Bang => self.is_ticking = !self.is_ticking,
                    Bang::BangBool(val) => self.is_ticking = *val,
                    _ => ()
                }
            }
            if let Some(ref bang) = inputs.get(1) {
                match bang {
                    Bang::BangF32(val) => self.clock_rate = Duration::from_secs_f32(*val),
                    Bang::BangU32(val) => self.clock_rate = Duration::from_secs(*val as u64),
                    _ => ()
                }
            }
            if !self.is_ticking {
                return;
            }
            if self.time_ellapsed > self.clock_rate {
                *output = Bang::Bang;
                self.time_ellapsed = Duration::from_secs(0);
            }
            else {
                *output = Bang::Empty;
            }
            let delta_time = N as f32 / self.sample_rate;
            self.time_ellapsed += Duration::from_secs_f32(delta_time);
        })
    }
}