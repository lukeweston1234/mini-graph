use crate::buffer::*;
use crate::node::*;

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
    // I am open to suggestions, to me it seems like a somewhat safe operation
    // as the double buffer size is known, not taking user input, etc.
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
    // safe version
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