use assert_no_alloc::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, BuildStreamError, FromSample, SampleRate, SizedSample, StreamConfig};
use oscillator::{Oscillator};

use crate::oscillator::{AudioPipeline, PipelineNode};

mod oscillator;

/// Settings up the output stream with the given config. This was heavily "inspired" by 
/// some of the examples on FunDSP.
fn run<const N: usize, T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<(), BuildStreamError>
where
    T: SizedSample + FromSample<f64>,
{
    let triangle_wave = Oscillator::new(440.0, SAMPLE_RATE as f32, 0.0, oscillator::Wave::SinWave);

    let triangle_wave_enum = PipelineNode::OscillatorNode(triangle_wave);

    let mut pipeline = AudioPipeline::new(vec![triangle_wave_enum]);

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


// / The function that takes an input from the audio pipeline, 
// / and delivers it to the CPAL slice. The CPAL slice is a 
// / frame of a certain buffer size. If you request a buffer size of 256,
// / with 2 channels, the output will have a length of 512. This function
// / also takes ownership of the audio pipeline.
fn write_data<const BUFFER_SIZE: usize, const CHANNEL_COUNT: usize, T>(
    output: &mut [T],
    audio_pipeline: &mut AudioPipeline<BUFFER_SIZE, CHANNEL_COUNT>,
)
where
    T: SizedSample + FromSample<f64>,
{    
    let next_pipeline_buffer = audio_pipeline.next_frame();

    for (frame_index, frame) in output.chunks_mut(CHANNEL_COUNT).enumerate() {
        for (channel, sample) in frame.iter_mut().enumerate() {
            let pipeline_next_frame = &next_pipeline_buffer[channel];
            *sample = T::from_sample(pipeline_next_frame[frame_index] as f64);
        }
    }
}

// fn write_data<const BUFFER_SIZE: usize, const CHANNEL_COUNT: usize, T>(
//     output: &mut [T],
// )
// where
//     T: SizedSample + FromSample<f64>,
// {
//     // Test 1: Just fill with silence - is this fast?
//     // for sample in output.iter_mut() {
//     //     *sample = T::from_sample(0.0);
//     // }
    
//     // Test 2: Skip pipeline processing - just use a simple oscillator
//     static mut PHASE: f32 = 0.0;
//     for chunk in output.chunks_mut(CHANNEL_COUNT) {
//         unsafe {
//             let sample = (PHASE * 2.0 * std::f32::consts::PI).sin() * 0.1;
//             PHASE += 440.0 / 48000.0;
//             if PHASE >= 1.0 { PHASE -= 1.0; }
            
//             for s in chunk {
//                 *s = T::from_sample(sample as f64);
//             }
//         }
//     }
// }

const CHANNEL_COUNT: usize = 2;
const FRAME_SIZE: usize = 1024;
const SAMPLE_RATE: u32 = 48_000;

fn main() {
    let host = cpal::default_host();

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