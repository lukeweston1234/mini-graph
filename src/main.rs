use assert_no_alloc::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, BuildStreamError, FromSample, SampleRate, SizedSample, StreamConfig};
use jack_demo::audio_graph::AudioGraph;
use jack_demo::graph::Graph;
use jack_demo::oscillator::{Oscillator, Wave};
use jack_demo::pipeline::AudioPipeline;
use jack_demo::{audio_graph, graph, params::*};
use jack_demo::adsr::ADSR;
use jack_demo::stream::write_data;


/// Settings up the output stream with the given config. This was heavily "inspired" by 
/// some of the examples on FunDSP.
fn run<const N: usize, T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<(), BuildStreamError>
where
    T: SizedSample + FromSample<f64>,
{
    let triangle_wave = Oscillator::new(440.0, SAMPLE_RATE, 0.0, Wave::SinWave);

    let trig = ParamBool::new(true);

    let trig_param = trig.clone();

    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(12));
        trig_param.store(false);
    });

    let adsr = ADSR::<FRAME_SIZE> { 
        attack: ParamF32::new(4.0), 
        sustain: ParamF32::new(0.3), 
        decay: ParamF32::new(4.0), 
        delta_time: ParamF32::new(0.0), 
        amplitude_scalar: ParamF32::new(0.0), 
        sample_rate: ParamU32::new(config.sample_rate.0), 
        channels: ParamU32::new(2), 
        delta_release_time: ParamF32::new(0.0), 
        release: ParamF32::new(4.0),
        trig: trig
    };

    let mut audio_graph = AudioGraph::<FRAME_SIZE, CHANNEL_COUNT>::new(8);

    audio_graph.add_node(Box::new(triangle_wave));
    audio_graph.add_node(Box::new(adsr));

    audio_graph.add_edge(0, 1);
    
    // let mut pipeline = AudioPipeline::new(vec![Box::new(triangle_wave), Box::new(adsr)]);

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



const CHANNEL_COUNT: usize = 2;
const FRAME_SIZE: usize = 1024;
const SAMPLE_RATE: u32 = 48_000;

fn main() {
    // let host = cpal::default_host();

    let host = cpal::host_from_id(cpal::HostId::Jack)
        .expect("JACK host not available");

    let device = host.default_output_device().unwrap();

    let config = StreamConfig {
        channels: CHANNEL_COUNT as u16,
        sample_rate: SampleRate(SAMPLE_RATE),
        buffer_size: BufferSize::Fixed(FRAME_SIZE as u32),
    };

    println!("{:?}", config);

    run::<FRAME_SIZE, f32>(&device, &config.into()).unwrap();
}