use crate::{buffer::{Buffer, Frame}, graph::{Graph}, node::Node};

type AudioGraphNode<const N: usize >= Box<dyn Node<N> + Send>;

pub struct AudioGraph<const BUFFER_SIZE: usize, const CHANNEL_COUNT: usize> {
    graph: Graph<AudioGraphNode<BUFFER_SIZE>>,
    output_buffers: Vec<Frame<BUFFER_SIZE, CHANNEL_COUNT>>,
    sort_order: Option<Vec<usize>>,
    work_buffer: Frame<BUFFER_SIZE, CHANNEL_COUNT>
}
impl<const BUFFER_SIZE: usize, const CHANNEL_COUNT: usize> AudioGraph<BUFFER_SIZE, CHANNEL_COUNT>{
    pub fn new(max_graph_size: usize) -> Self {
        Self {
            graph: Graph::with_capacity(max_graph_size),
            output_buffers: vec![std::array::from_fn(|_| Buffer::<BUFFER_SIZE>::default()) ;max_graph_size],
            sort_order: None,
            work_buffer: std::array::from_fn(|_| Buffer::<BUFFER_SIZE>::default())
        }
    }
    pub fn add_node(&mut self, node: AudioGraphNode<BUFFER_SIZE>){
        self.graph.add_node(node);
    }
    pub fn add_edge(&mut self, source_index: usize, target_index: usize){
        if let Ok(_) = self.graph.add_edge(source_index, target_index) {
            self.invalidate_sort_order();
        }
    }
    fn invalidate_sort_order(&mut self){
        if let Ok(sort_order) = self.graph.topo_sort(){
            self.sort_order = Some(sort_order);
            println!("{:?}", self.sort_order)
        }
    }
    #[inline(always)]
    pub fn next_frame(&mut self) -> &Frame<BUFFER_SIZE, CHANNEL_COUNT>{
        // Sin,  ADSR, Reverb, Compressor
        let buf_ptr = self.output_buffers.as_mut_ptr();
        if let Some(sort_order) = &self.sort_order {
            for &node_idx in sort_order.iter() {
                let (node, targets) = self.graph.get_node_and_targets_mut(node_idx);
    
                let read_slice: &[Buffer<BUFFER_SIZE>] =
                    unsafe { &*buf_ptr.add(node_idx) };

                node.process(read_slice, &mut self.work_buffer);

                for &tgt in targets {
                    self.output_buffers[tgt].copy_from_slice(&self.work_buffer);
                }
            }
        }
        &self.work_buffer
    }
}