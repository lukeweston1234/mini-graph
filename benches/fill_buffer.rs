// benches/write_data.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jack_demo::oscillator::{Oscillator, Wave};
use jack_demo::pipeline::AudioPipeline;
use jack_demo::params::*;
use jack_demo::adsr::ADSR;
use jack_demo::stream::write_data;

const CHANNEL_COUNT: usize = 2;
const FRAME_SIZE: usize = 512;
const SAMPLE_RATE: u32 = 48_000;

fn make_pipeline() -> AudioPipeline<FRAME_SIZE, CHANNEL_COUNT> {
    let osc = Oscillator::new(440.0, SAMPLE_RATE as f32, 0.0, Wave::SinWave);

    let trig = ParamBool::new(true);
    let adsr = ADSR {
        attack: ParamF32::new(4.0),
        sustain: ParamF32::new(0.3),
        decay: ParamF32::new(4.0),
        delta_time: ParamF32::new(0.0),
        amplitude_scalar: ParamF32::new(0.0),
        sample_rate: ParamU32::new(SAMPLE_RATE),
        channels: ParamU32::new(CHANNEL_COUNT as u32),
        delta_release_time: ParamF32::new(0.0),
        release: ParamF32::new(4.0),
        trig: trig.clone(),
    };

    AudioPipeline::new(vec![Box::new(osc), Box::new(adsr)])
}

fn bench_fill_buffer(c: &mut Criterion) {
    let mut buffer = vec![0f32; FRAME_SIZE * CHANNEL_COUNT];
    let mut pipeline = make_pipeline();

    c.bench_function("write_data", |b| {
        b.iter(|| {
            write_data::<FRAME_SIZE, CHANNEL_COUNT, f32>(
                black_box(&mut buffer),
                black_box(&mut pipeline),
            );
            black_box(buffer[0])
        })
    });
}

criterion_group!(benches, bench_fill_buffer);
criterion_main!(benches);
