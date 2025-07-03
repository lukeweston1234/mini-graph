use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mini_graph::buffer::{self, Buffer};
use mini_graph::mixer::Mixer;
use mini_graph::osc::{Oscillator, Wave};
use mini_graph::audio_graph::AudioGraph;
use mini_graph::write::write_data;


const CHANNEL_COUNT: usize = 2;
const FRAME_SIZE: usize = 512;
const SAMPLE_RATE: u32 = 48_000;

fn make_graph() -> AudioGraph<FRAME_SIZE, CHANNEL_COUNT> {
    let mut audio_graph = AudioGraph::<FRAME_SIZE, CHANNEL_COUNT>::with_capacity(16);

    let id_0 = audio_graph.add_node(Box::new(
        Oscillator::new(261.63, SAMPLE_RATE, 0.0, Wave::SinWave)
    ));
    let id_1 = audio_graph.add_node(Box::new(
        Oscillator::new(493.88, SAMPLE_RATE, 0.0, Wave::SinWave)
    ));
    let id_2 = audio_graph.add_node(Box::new(
        Oscillator::new(392.00, SAMPLE_RATE, 0.0, Wave::SinWave)
    ));
    let id_3 = audio_graph.add_node(Box::new(
        Oscillator::new(329.63, SAMPLE_RATE, 0.0, Wave::SinWave)
    ));

    let mix_id = audio_graph.add_node(Box::new(Mixer::default()));

    audio_graph.add_edge(id_0, mix_id);
    audio_graph.add_edge(id_1, mix_id);
    audio_graph.add_edge(id_2, mix_id);
    audio_graph.add_edge(id_3, mix_id);

    audio_graph.set_sink_index(mix_id);

    audio_graph
}

fn bench_write_chord(c: &mut Criterion){
    let mut buffer = Buffer::<FRAME_SIZE>::default();
    let mut graph = make_graph();

    c.bench_function("write_to_buffer", |b| {
        b.iter(|| {
            write_data(&mut buffer, &mut graph);
        });
    });
}

criterion_group!(benches, bench_write_chord);
criterion_main!(benches);
