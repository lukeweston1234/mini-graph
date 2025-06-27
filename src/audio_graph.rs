use crate::{buffer::{Buffer, Frame}, graph::{Graph, NodeIndex}, node::Node};

type AudioGraphInner<const N: usize> = Graph<AudioGraphNode<N>>;
type AudioGraphNode<const N: usize >= Box<dyn Node<N> + Send>;

pub struct AudioGraph<const BUFFER_SIZE: usize, const CHANNEL_COUNT: usize> {
    graph: AudioGraphInner<BUFFER_SIZE>,
    work_buffers: Vec<Frame<BUFFER_SIZE, CHANNEL_COUNT>>,
    max_graph_size: usize,
    sort_order: Option<Vec<NodeIndex>>,
    output_node_index: Option<usize>,
}
impl<const BUFFER_SIZE: usize, const CHANNEL_COUNT: usize> AudioGraph<BUFFER_SIZE, CHANNEL_COUNT>{
    pub fn new(max_graph_size: usize) -> Self {
        Self {
            graph: Graph::new(max_graph_size),
            work_buffers: Vec::with_capacity(max_graph_size),
            max_graph_size: max_graph_size,
            sort_order: None,
            output_node_index: None
        }
    }
    pub fn add_node(&mut self, node: AudioGraphNode<BUFFER_SIZE>){
        self.graph.add_node(node);
    }
    pub fn add_edge(&mut self, source_index: usize, target_index: usize){
        self.graph.add_edge(source_index, target_index);
        self.invalidate_sort_order();
    }
    fn invalidate_sort_order(&mut self){
        self.sort_order = self.graph.topo_sort();
    }
}
impl<const BUFFER_SIZE: usize, const CHANNEL_COUNT: usize> Node<BUFFER_SIZE> for AudioGraph<BUFFER_SIZE, CHANNEL_COUNT>{
    fn process(&mut self, _: &[Buffer<BUFFER_SIZE>], output: &mut [Buffer<BUFFER_SIZE>]) {
        // OSC -> ADSR -> Reverb
        // OSC.process(input, output);
        // ADSR.process(input, output);
        // Reverb.process(input, output);

        if let Some(sort_order) = &mut self.sort_order {
            
        }
        
    }
}
