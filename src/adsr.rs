use crate::node::{Node, Bang};
use crate::buffer::Frame;
use crate::math::lerp;
pub struct ADSR<const FRAME_SIZE: usize, const CHANNELS: usize>{
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    // delta_time parameters for time elapsed
    delta_time: f32,
    delta_release_time: f32,

    amplitude_scalar: f32,
    gate: bool,

    sample_rate: f32,
}
impl<const N: usize, const C: usize> ADSR<N, C> {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            attack: 4.0,
            sustain: 3.0,
            decay: 4.0,
            release:5.0,
            delta_release_time: 0.0,
            sample_rate: sample_rate as f32,
            delta_time: 0.0,
            amplitude_scalar: 0.0,
            gate: false,
        }
    }
}

impl<const N: usize, const C: usize> Node<N, C> for  ADSR<N, C> {
    fn process(&mut self, inputs: &[Frame<N, C>], output: &mut Frame<N, C>) {
        let input = inputs[0];
        let mut volume = 0.0;
        
        for n in 0..N {
            if self.delta_time < self.attack {
                volume = lerp(0.0, 1.0, self.delta_time / self.attack);
            }
            else if self.gate {
                let decay_delta = self.delta_time - self.attack;
    
                if decay_delta < self.decay {
                    volume = lerp(1.0, self.sustain, decay_delta / self.decay as f32);
                }
                else {
                    volume = self.sustain;
                }
                self.amplitude_scalar = volume;
            }
            else {
                if self.delta_release_time < self.release {
                    volume = lerp(self.amplitude_scalar, 0.0, self.delta_release_time / self.release as f32);
                }
                else {
                    volume = 0.0;
                }
                if self.delta_release_time < self.release{
                    let inc_time = (1.0) / self.sample_rate as f32;
                    self.delta_release_time += inc_time;
                }
            }
                
            self.delta_time += 1.0 / self.sample_rate;

            for c in 0..C {
                output[c][n] = input[c][n] * volume;
            }
        }
    }
    fn handle_bang(&mut self, inputs: &[Bang], _: &mut Bang) {
        let res = inputs.get(0);
        if let Some(bang) = res {
            match *bang {
                Bang::Bang => self.gate = !self.gate,
                Bang::BangBool(val) => self.gate = val,
                _ => (),
            }
        }
    }
}
