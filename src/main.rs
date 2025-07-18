use mini_graph::delay_line::DelayLine;
use mini_graph::gain::Gain;
use mini_graph::mixer::Mixer;
use mini_graph::osc::{Oscillator, Wave};
use mini_graph::write::*;
use mini_graph::audio_graph::{DynamicAudioGraph};
use assert_no_alloc::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, BuildStreamError, FromSample, SampleRate, SizedSample, StreamConfig};


#[cfg(debug_assertions)] // required when disable_release is set (default)
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

const SAMPLE_RATE: u32 = 48_000;
const FRAME_SIZE: usize = 1024;
const CHANNEL_COUNT: usize = 2;

fn run<const N: usize, T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<(), BuildStreamError>
where
    T: SizedSample + FromSample<f64>,
{
    let mut audio_graph = DynamicAudioGraph::<FRAME_SIZE, CHANNEL_COUNT>::with_capacity(16);
    
    let master_id = audio_graph.add_node(Box::new(Mixer::default()));

    // ─── Oscillators ────────────────────────────────────────────────────────────────
    // 1 (C₄), 7 (B₄), 5 (G₄), 3 (E₄)
    let osc_1 = audio_graph.add_node(Box::new(
        Oscillator::new(261.63, SAMPLE_RATE, 0.0, Wave::SinWave) // C₄
    ));
    let osc_2 = audio_graph.add_node(Box::new(
        Oscillator::new(493.88, SAMPLE_RATE, 0.0, Wave::SinWave) // B₄
    ));
    let osc_3 = audio_graph.add_node(Box::new(
        Oscillator::new(392.00, SAMPLE_RATE, 0.0, Wave::SinWave) // G₄
    ));
    let osc_4 = audio_graph.add_node(Box::new(
        Oscillator::new(329.63, SAMPLE_RATE, 0.0, Wave::SinWave) // E₄
    ));

    // ─── Mixer ───────────────────────────────────────────────────────────────────
    let chord_bus = audio_graph.add_node(Box::new(Mixer::default()));

    let gain_id = audio_graph.add_node(Box::new(Gain::new(0.8))); // Some clipping limited to -1, 1

    let delay_id = audio_graph.add_node(Box::new(DelayLine::new(12000)));

    let delay_gain_id = audio_graph.add_node(Box::new(Gain::new(0.8)));


    audio_graph.add_edges(&[(osc_1, chord_bus),(osc_2, chord_bus),(osc_3, chord_bus),(osc_4,chord_bus), (chord_bus, gain_id), (gain_id, master_id)]);

    audio_graph.add_edges(&[(chord_bus, delay_id), (delay_id, delay_gain_id), (delay_gain_id, master_id)]);
    // ─── Sink ─────────────────────────────────────────────────────────────────────
    audio_graph.set_sink_index(master_id);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            assert_no_alloc( || write_data::<FRAME_SIZE, CHANNEL_COUNT, f32>(data, &mut audio_graph))
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