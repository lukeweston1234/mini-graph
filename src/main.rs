mod buffer;
mod stream;

use std::collections::VecDeque;
use indexmap::IndexSet;
use crate::buffer::{Buffer, Frame};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, BuildStreamError, FromSample, SampleRate, SizedSample, StreamConfig};

const SAMPLE_RATE: u32 = 48_000;
const FRAME_SIZE: usize = 1024;
const CHANNEL_COUNT: usize = 2;


// / The function that takes an input from the audio pipeline, 
// / and delivers it to the CPAL slice. The CPAL slice is a 
// / frame of a certain buffer size. If you request a buffer size of 256,
// / with 2 channels, the output will have a length of 512. This function
// / also takes ownership of the audio pipeline.
#[inline(always)]
pub fn write_data<const BUFFER_SIZE: usize, const CHANNEL_COUNT: usize, T>(
    output: &mut [T],
    audio_graph: &mut AudioGraph<BUFFER_SIZE, CHANNEL_COUNT>,
)
where
    T: SizedSample + FromSample<f64>,
{    
    
    let next_pipeline_buffer = audio_graph.next_block();

    for (frame_index, frame) in output.chunks_mut(CHANNEL_COUNT).enumerate() {
        for (channel, sample) in frame.iter_mut().enumerate() {
            let pipeline_next_frame = &next_pipeline_buffer[channel];
            *sample = T::from_sample(pipeline_next_frame[frame_index] as f64);
        }
    }
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
    pub fn new(freq: f32, sample_rate: u32, phase: f32, wave: Wave) -> Self {
        Self {
            freq,
            sample_rate: sample_rate as f32,
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
        self.phase += self.freq / self.sample_rate as f32;
        self.phase -= (self.phase >= 1.0) as u32 as f32; 
        sample
    }
}
impl<const N: usize, const C: usize> Node<N, C> for Oscillator<N> {
    fn process(&mut self, _: &[Frame<N, C>], outputs: &mut Frame<N, C>){
        for i in 0..N {
            let sample = self.tick_osc();
            for buf in outputs.iter_mut() {
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

trait Node<const N: usize, const C: usize> {
    fn process(&mut self, inputs: &[Frame<N, C>], outputs: &mut Frame<N, C>){}
}

type BoxedNode<const N: usize, const C: usize> = Box<dyn Node<N, C> + Send> ;

pub enum GraphError {
    CycleDetected
}

struct AudioGraph<const BUFFER_SIZE: usize, const CHANNEL_COUNT: usize> {
    nodes: Vec<BoxedNode<BUFFER_SIZE, CHANNEL_COUNT>>,
    inputs: Vec<IndexSet<usize>>,
    outputs: Vec<IndexSet<usize>>,
    output_buffers: Vec<Frame<BUFFER_SIZE, CHANNEL_COUNT>>,
    sort_order: Vec<usize>,
    sink_index: usize,
}
impl<const BUFFER_SIZE: usize, const CHANNEL_COUNT: usize> AudioGraph<BUFFER_SIZE, CHANNEL_COUNT> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
            inputs: vec![IndexSet::with_capacity(capacity);capacity],
            outputs: vec![IndexSet::with_capacity(capacity);capacity],
            output_buffers: vec![[Buffer::<BUFFER_SIZE>::default(); CHANNEL_COUNT]; capacity],
            sort_order: Vec::with_capacity(capacity),
            sink_index: 0,
        }
    }
    pub fn add_node(&mut self, node: BoxedNode<BUFFER_SIZE, CHANNEL_COUNT>) -> usize {
        let id = self.nodes.len();
        self.nodes.push(node);
        id
    }
    pub fn add_edge(&mut self, from: usize, to: usize) {
        self.inputs[to].insert(from);
        self.outputs[from].insert(to);
        self.invalidate_sort_order();
    }
    pub fn set_sink_index(&mut self, index: usize){
        self.sink_index = index;
    }
    fn invalidate_sort_order(&mut self) {
        if let Ok(topo) = self.topo_sort() {
            self.sort_order = topo;
        }
        else {
            panic!("Cycle detected")
        }
    }
    fn topo_sort(&self) -> Result<Vec<usize>, GraphError> {
        let mut indegree: Vec<usize> = vec![0; self.nodes.len()];

        for targets in &self.outputs {
            for target in targets {
                indegree[*target] += 1;
            }
        }

        let mut no_incoming_edges_queue = VecDeque::new();
        for (index, count) in indegree.iter().enumerate() {
            if *count == 0 {
                no_incoming_edges_queue.push_back(index);
            }
        }


        let mut sorted: Vec<usize> = Vec::with_capacity(self.nodes.len());
        while let Some(node_index) = no_incoming_edges_queue.pop_front() {
            sorted.push(node_index);
            if let Some(connections) = self.outputs.get(node_index){
                for v_id in connections {
                    indegree[*v_id] -= 1;
                    if indegree[*v_id] == 0 {
                        no_incoming_edges_queue.push_back(*v_id);
                    }
                }
            }
        }

        if sorted.len() == indegree.len() {
            Ok(sorted)
        }
        else {
            Err(GraphError::CycleDetected)
        }
    }
    #[inline(always)]
    pub fn next_block(&mut self) -> &Frame<BUFFER_SIZE, CHANNEL_COUNT>{
        for index in self.sort_order.iter() {
            let node = &mut self.nodes[*index];
            let input_indexes = &self.inputs[*index];

            let inputs: Vec<Frame<BUFFER_SIZE, CHANNEL_COUNT>> = input_indexes.iter().map(|i| self.output_buffers[*i]).collect();

            node.process(inputs.as_slice(), &mut self.output_buffers[*index]);
        }

        &self.output_buffers[self.sink_index]
    }
}


fn run<const N: usize, T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<(), BuildStreamError>
where
    T: SizedSample + FromSample<f64>,
{
    let mut audio_graph = AudioGraph::<FRAME_SIZE, CHANNEL_COUNT>::with_capacity(16);

    // ─── Oscillators ────────────────────────────────────────────────────────────────
    // 1 (C₄), 7 (B₄), 5 (G₄), 3 (E₄)
    let id_0 = audio_graph.add_node(Box::new(
        Oscillator::new(261.63, SAMPLE_RATE, 0.0, Wave::SinWave) // C₄
    ));
    let id_1 = audio_graph.add_node(Box::new(
        Oscillator::new(493.88, SAMPLE_RATE, 0.0, Wave::SinWave) // B₄
    ));
    let id_2 = audio_graph.add_node(Box::new(
        Oscillator::new(392.00, SAMPLE_RATE, 0.0, Wave::SinWave) // G₄
    ));
    let id_3 = audio_graph.add_node(Box::new(
        Oscillator::new(329.63, SAMPLE_RATE, 0.0, Wave::SinWave) // E₄
    ));

    // ─── Mixer ───────────────────────────────────────────────────────────────────
    let mix_id = audio_graph.add_node(Box::new(Mixer::default()));

    // ─── Wire them ────────────────────────────────────────────────────────────────
    audio_graph.add_edge(id_0, mix_id);
    audio_graph.add_edge(id_1, mix_id);
    audio_graph.add_edge(id_2, mix_id);
    audio_graph.add_edge(id_3, mix_id);

    // ─── Sink ─────────────────────────────────────────────────────────────────────
    audio_graph.set_sink_index(mix_id);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // assert_no_alloc( || write_data::<FRAME_SIZE, CHANNEL_COUNT, f32>(data, &mut audio_graph))
            write_data::<FRAME_SIZE, CHANNEL_COUNT, f32>(data, &mut audio_graph)
        },
        |err| eprintln!("An output stream error occured: {}", err),
        None,
    )?;

    stream.play().unwrap();

    std::thread::park();

    Ok(())
}


fn main(){
    
    let host = cpal::host_from_id(cpal::HostId::Jack)
    .expect("JACK host not available");

    let device = host.default_output_device().unwrap();

    let config = StreamConfig {
        channels: CHANNEL_COUNT as u16,
        sample_rate: SampleRate(SAMPLE_RATE),
        buffer_size: BufferSize::Fixed(FRAME_SIZE as u32),
    };

    run::<FRAME_SIZE, f32>(&device, &config.into()).unwrap();

    std::thread::park();
}



#[cfg(test)]
mod tests {
    use super::*;
}
