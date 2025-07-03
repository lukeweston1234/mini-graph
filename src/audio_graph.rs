use std::collections::VecDeque;
use indexmap::IndexSet;
use super::buffer::{Buffer, Frame};
use super::node::*;

pub enum GraphError {
    CycleDetected
}

pub struct AudioGraph<const BUFFER_SIZE: usize, const CHANNEL_COUNT: usize> {
    nodes: Vec<BoxedNode<BUFFER_SIZE, CHANNEL_COUNT>>,
    inputs: Vec<IndexSet<usize>>,
    outputs: Vec<IndexSet<usize>>,
    output_buffers: Vec<Frame<BUFFER_SIZE, CHANNEL_COUNT>>,
    sort_order: Vec<usize>,
    sink_index: usize,
}
impl<const BUFFER_SIZE: usize, const CHANNEL_COUNT: usize> AudioGraph<BUFFER_SIZE, CHANNEL_COUNT> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
            inputs: vec![IndexSet::with_capacity(capacity);capacity],
            outputs: vec![IndexSet::with_capacity(capacity);capacity],
            output_buffers: vec![[Buffer::<BUFFER_SIZE>::default(); CHANNEL_COUNT]; capacity],
            sort_order: Vec::with_capacity(capacity),
            sink_index: 0,
        }
    }
    pub fn add_node(&mut self, node: BoxedNode<BUFFER_SIZE, CHANNEL_COUNT>) -> usize {
        let id = self.nodes.len();
        self.nodes.push(node);
        id
    }
    pub fn add_edge(&mut self, from: usize, to: usize) {
        self.inputs[to].insert(from);
        self.outputs[from].insert(to);
        self.invalidate_sort_order();
    }
    pub fn set_sink_index(&mut self, index: usize){
        self.sink_index = index;
    }
    fn invalidate_sort_order(&mut self) {
        if let Ok(topo) = self.topo_sort() {
            self.sort_order = topo;
        }
        else {
            panic!("Cycle detected")
        }
    }
    fn topo_sort(&self) -> Result<Vec<usize>, GraphError> {
        let mut indegree: Vec<usize> = vec![0; self.nodes.len()];

        for targets in &self.outputs {
            for target in targets {
                indegree[*target] += 1;
            }
        }

        let mut no_incoming_edges_queue = VecDeque::new();
        for (index, count) in indegree.iter().enumerate() {
            if *count == 0 {
                no_incoming_edges_queue.push_back(index);
            }
        }


        let mut sorted: Vec<usize> = Vec::with_capacity(self.nodes.len());
        while let Some(node_index) = no_incoming_edges_queue.pop_front() {
            sorted.push(node_index);
            if let Some(connections) = self.outputs.get(node_index){
                for v_id in connections {
                    indegree[*v_id] -= 1;
                    if indegree[*v_id] == 0 {
                        no_incoming_edges_queue.push_back(*v_id);
                    }
                }
            }
        }

        if sorted.len() == indegree.len() {
            Ok(sorted)
        }
        else {
            Err(GraphError::CycleDetected)
        }
    }
    #[inline(always)]
    pub fn next_block(&mut self) -> &Frame<BUFFER_SIZE, CHANNEL_COUNT>{
        for index in self.sort_order.iter() {
            let node = &mut self.nodes[*index];
            let input_indexes = &self.inputs[*index];

            let inputs: Vec<Frame<BUFFER_SIZE, CHANNEL_COUNT>> = input_indexes.iter().map(|i| self.output_buffers[*i]).collect();

            node.process(inputs.as_slice(), &mut self.output_buffers[*index]);
        }

        &self.output_buffers[self.sink_index]
    }
}