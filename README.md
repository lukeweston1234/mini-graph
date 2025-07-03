# mini-graph

This repo serves mostly as a learning exercise for structuring larger projects, or a less opinionated audio graph framework for rolling your own nodes. For something more feature complete, I would suggest FunDSP, which has support for things like SIMD instructions.

The general DX revolves around the creation of nodes that implement the process trait, which lets users quickly build a graph of heap allocated nodes. For audio purposes, I would suggest allocating these either in a seperate thread of before your audio thread is started, in order to avoid any pops or cracks from missed audio frames. These nodes and edges are then topologically sorted, so that their dependencies compute before them. Each node then takes all of it's inputs, and writes to its associated output buffer. There is finally a sink index, that CPAL can pull from. 

I am planning on writing a MIDI node system or something similar to PureData's midi building blocks, and I am also looking at eventually adding SIMD support or mutlithreading(perhaps computing graph branches with no shared dependencies?), although these are considerations for the future. 

### Example Node

You can define a basic gain node like so:

```rust
use crate::node::Node;
use crate::buffer::Frame;

pub struct Gain<const FRAME_SIZE: usize> {
    gain: f32 // Arc<AtomicF32> might be more helpful. If you need an atomic f32 there is an easy trick
}
impl<const N: usize> Gain<N> {
    pub fn new(gain: f32) -> Self {
        Self {
            gain
        }
    }
}
impl <const N: usize, const C: usize> Node<N, C> for Gain<N> {
    fn process(&mut self, inputs: &[Frame<N, C>], output: &mut Frame<N, C>){
        // This node only takes an input of one stereo buffer.
        let input = inputs[0];
        for n in 0..N { // For ever sample in our frame size
            for c in 0..C { // For ever channel in our frame
                output[c][n] = (input[c][n] * self.gain).clamp(-1.0 , 1.0);
            }
        }
    }
}
```