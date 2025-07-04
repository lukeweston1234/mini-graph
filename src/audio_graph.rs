pub trait AudioGraph<const BUFFER_SIZE: usize, const CHANNEL_COUNT: usize> {
    fn next_block(&mut self) -> &Frame<BUFFER_SIZE, CHANNEL_COUNT>;
    fn invalidate_sort_order(&mut self);
}

use super::buffer::{Buffer, Frame};
use super::node::BoxedNode;
use super::graph::{DynamicGraph, Graph};

pub struct DynamicAudioGraph<const BUFFER_SIZE: usize, const CHANNEL_COUNT: usize> {
    graph: DynamicGraph<BoxedNode<BUFFER_SIZE, CHANNEL_COUNT>>,
    inputs_buffer: Vec<Frame<BUFFER_SIZE, CHANNEL_COUNT>>,
    output_buffers: Vec<Frame<BUFFER_SIZE, CHANNEL_COUNT>>,
    sort_order: Vec<usize>,
    sink_index: usize,
}

impl<const BUFFER_SIZE: usize, const CHANNEL_COUNT: usize> DynamicAudioGraph<BUFFER_SIZE, CHANNEL_COUNT> {
    pub fn with_capacity(capacity: usize) -> Self {
        let graph = DynamicGraph::with_capacity(capacity);
        Self {
            graph,
            inputs_buffer: Vec::with_capacity(capacity),
            output_buffers: vec![[Buffer::<BUFFER_SIZE>::default(); CHANNEL_COUNT]; capacity],
            sort_order: Vec::with_capacity(capacity),
            sink_index: 0,
        }
    }

    pub fn add_node(&mut self, node: BoxedNode<BUFFER_SIZE, CHANNEL_COUNT>) -> usize {
        let id = self.graph.add_node(node);
        id
    }

    pub fn add_edge(&mut self, from: usize, to: usize) {
        self.graph.add_edge(from, to);
        self.invalidate_sort_order();
    }

    pub fn add_edges(&mut self, edges: &[(usize, usize)]) {
        self.graph.add_edges(edges);
        self.invalidate_sort_order();
    }

    pub fn set_sink_index(&mut self, sink: usize) {
        self.sink_index = sink;
    }

    fn invalidate_sort_order(&mut self) {
        match self.graph.topo_sort() {
            Ok(order) => self.sort_order = order,
            Err(_) => panic!("Cycle detected in audio graph"),
        }
    }

    #[inline(always)]
    pub fn next_block(&mut self) -> &Frame<BUFFER_SIZE, CHANNEL_COUNT> {
        for &node_index in &self.sort_order {
            let node = &mut self.graph.nodes[node_index];
            let inputs = &self.graph.incoming[node_index];

            self.inputs_buffer.clear();
            self.inputs_buffer.reserve(inputs.len());
            for &src in inputs {
                self.inputs_buffer.push(self.output_buffers[src]);
            }

            node.process(&self.inputs_buffer, &mut self.output_buffers[node_index]);
        }

        &self.output_buffers[self.sink_index]
    }
}
