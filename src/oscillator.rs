use crate::adsr::ADSR;
use crate::buffer::{Buffer, Frame};

pub trait Node<const N: usize> {
    fn process(&mut self, input: &[Buffer<N>], output: &mut [Buffer<N>]);
}

pub enum Wave {
    SinWave,
    SawWave,
    TriangleWave,
    SquareWave,
}

pub struct Oscillator<const BUFFER_SIZE: usize> {
    freq: f32,
    sample_rate: f32,
    phase: f32,
    wave: Wave
}

impl<const N: usize> Oscillator<N> {
    pub fn new(freq: f32, sample_rate: f32, phase: f32, wave: Wave) -> Self {
        Self {
            freq,
            sample_rate,
            phase,
            wave
        }
    }
    pub fn set_wave_form(&mut self, wave: Wave){
        self.wave = wave;
    }
    #[inline]
    fn tick_osc(&mut self) -> f32 {
        let sample = match self.wave {
            Wave::SinWave => sin_amp_from_phase(&self.phase),
            Wave::SawWave => saw_amp_from_phase(&self.phase),
            Wave::SquareWave => square_amp_from_phase(&self.phase),
            Wave::TriangleWave => triangle_amp_from_phase(&self.phase),
        };
        self.phase += self.freq / self.sample_rate;
        self.phase -= (self.phase >= 1.0) as u32 as f32; 
        sample
    }
     fn process(&mut self, _: &[Buffer<N>], output: &mut [Buffer<N>]){
        for i in 0..N {
            let sample = self.tick_osc();
            for buf in output.iter_mut() {
                buf[i] = sample;
            }
        }
    }
}

#[inline]
fn sin_amp_from_phase(phase: &f32) -> f32 {
    (*phase * 2.0 * std::f32::consts::PI).sin()
}

#[inline]
fn saw_amp_from_phase(phase: &f32) -> f32 {
    *phase * 2.0 - 1.0
}

#[inline]
fn triangle_amp_from_phase(phase: &f32) -> f32 {
    2.0 * ((-1.0 + (*phase * 2.0)).abs() - 0.5)
}

#[inline]
fn square_amp_from_phase(phase: &f32) -> f32 {
    match *phase <= 0.5 {
        true => 1.0,
        false => -1.0,
    }
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

/// Here we are running process on all of the nodes, in a pipeline format.
/// This was chose because it's simple, and allows us to move to a graph if 
/// we truly need it down the line. Additionally, the pipeline is expected to render
/// audio to the main thread in the requested channel count.
pub struct AudioPipeline<
    const BUFFER_SIZE: usize,
    const CHANNEL_COUNT: usize,
> {
    nodes: Vec<PipelineNode<BUFFER_SIZE>>,
    bufs: [Frame<BUFFER_SIZE, CHANNEL_COUNT>; 2],
    idx: usize, // 0 = use bufs[0] as “in”, 1 = use bufs[1]
}

impl<const BUFFER_SIZE: usize, const CHANNEL_COUNT: usize>
    AudioPipeline<BUFFER_SIZE, CHANNEL_COUNT>
{
    pub fn new(nodes: Vec<PipelineNode<BUFFER_SIZE>>) -> Self {
        let bufs = std::array::from_fn(|_| std::array::from_fn(|_| Buffer::<BUFFER_SIZE>::default()));
        Self {
            nodes,
            bufs,
            idx: 0,
        }
    }
    // This unsafe approach is around 3% faster than the version below it.
    // I am open to suggestions!
    #[inline(always)]
    pub fn next_frame(&mut self) -> &Frame<BUFFER_SIZE, CHANNEL_COUNT> {
        let ptr = self.bufs.as_mut_ptr();

        for node in &mut self.nodes {
            unsafe {
                let in_buf  = &mut *ptr.add(self.idx);
                let out_buf = &mut *ptr.add(self.idx ^ 1);
                node.process(in_buf, out_buf);
            }
            self.idx ^= 1;
        }

        &self.bufs[self.idx]
    }

    // #[inline(always)]
    // pub fn next_frame(&mut self) -> &Frame<BUFFER_SIZE, CHANNEL_COUNT> {
    //     for node in &mut self.nodes {
    //         let [a, b] = &mut self.bufs;
    //         let (in_buf, out_buf) = if self.idx == 0 {
    //             (a, b)
    //         } else {
    //             (b, a)
    //         };

    //         node.process(in_buf, out_buf);
    //         self.idx ^= 1;
    //     }
    //     &self.bufs[self.idx]
    // }
}