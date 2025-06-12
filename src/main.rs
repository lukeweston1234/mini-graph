use assert_no_alloc::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, BuildStreamError, FromSample, SampleRate, SizedSample, StreamConfig};
use jack_demo::oscillator::{AudioPipeline, PipelineNode, Oscillator, Wave};
use jack_demo::params::*;
use jack_demo::adsr::ADSR;
use jack_demo::stream::write_data;


/// Settings up the output stream with the given config. This was heavily "inspired" by 
/// some of the examples on FunDSP.
fn run<const N: usize, T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<(), BuildStreamError>
where
    T: SizedSample + FromSample<f64>,
{
    let triangle_wave = Oscillator::new(440.0, SAMPLE_RATE as f32, 0.0, Wave::SinWave);

    let triangle_wave_enum = PipelineNode::OscillatorNode(triangle_wave);

    let trig = ParamBool::new(true);

    let trig_param = trig.clone();

    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(12));
        trig_param.store(false);
    });

    let adsr = ADSR { 
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

    let adsr_enum = PipelineNode::ADSRNode(adsr);
    

    let mut pipeline = AudioPipeline::new(vec![triangle_wave_enum, adsr_enum]);

    // TODO: Audit if assert_no_alloc actually does anything, I seam to be able to alloc on the audio thread

    let stream = device.build_output_stream(
        config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            assert_no_alloc( || write_data::<FRAME_SIZE, CHANNEL_COUNT, f32>(data, &mut pipeline))
        },
        |err| eprintln!("An output stream error occured: {}", err),
        None,
    )?;

    stream.play().unwrap();

    // Block the main thread (or sleep forever, or do other work).
    // For example:
    std::thread::park();

    Ok(())
}



const CHANNEL_COUNT: usize = 2;
const FRAME_SIZE: usize = 512;
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

    println!("{:?}", config.buffer_size);

    run::<FRAME_SIZE, f32>(&device, &config.into()).unwrap();
}