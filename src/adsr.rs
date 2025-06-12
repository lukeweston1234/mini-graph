use crate::oscillator::{Buffer, Node};
use crate::params::*;
use crate::math::lerp;

pub struct ADSR<const N: usize> {
    // These parameters all represent the attack and decay times, in seconds
    pub attack: ParamF32,
    pub decay: ParamF32,
    // The sustain mult.
    pub sustain: ParamF32,
    pub release: ParamF32,
    // delta_time is total time ellaphsed while triggered, release time is time since the trig is removed
    pub delta_time: ParamF32,
    pub delta_release_time: ParamF32,

    pub amplitude_scalar: ParamF32, // We need this when releasing, so we know where to lerp from
    pub trig: ParamBool,

    pub sample_rate: ParamU32,
    pub channels: ParamU32,
}



impl<const N: usize> Node<N> for ADSR<N> {
    fn process(&mut self, inputs: &[Buffer<N>], output: &mut [Buffer<N>]){
        for i in 0..N {
            let volume;
    
            if self.delta_time.get() < self.attack.get() {
                volume = lerp(0.0, 1.0, self.delta_time.get() / self.attack.get());
            }
            else if self.trig.get() {
                let decay_delta = self.delta_time.get() - self.attack.get();
    
                if decay_delta < self.decay.get() {
                    volume = lerp(1.0, self.sustain.get(), decay_delta / self.decay.get() as f32);
                }
                else {
                    volume = self.sustain.get();
                }
                self.amplitude_scalar.store(volume);
            }
            else {
                if self.delta_release_time.get() < self.release.get() {
                    volume = lerp(self.amplitude_scalar.get(), 0.0, self.delta_release_time.get() / self.release.get() as f32);
                }
    
                else {
                    volume = 0.0;
                }
    
                if self.delta_release_time.get() < self.release.get(){
                    let inc_time = (1.0) / self.sample_rate.get() as f32;
                    self.delta_release_time.add(inc_time);
                }
    
                // println!("amp_scalar: {:?}", self.amplitude_scalar.get());
    
                // println!("d_release_time: {:?}", self.delta_release_time.get());
    
                // panic!();
            }
                
            self.delta_time.add((1.0) / self.sample_rate.get() as f32);

            for (channels, buf) in output.iter_mut().enumerate(){
                buf[i] = inputs[channels][i] * volume;
            }
        }
    }
}