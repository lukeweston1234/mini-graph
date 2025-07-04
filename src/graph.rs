
// #![no_std]
// extern crate alloc;

use std::collections::VecDeque;

use indexmap::IndexSet;


pub enum GraphError {
    MaximumCapacity,
    CycleDetected
}

/// Our graph trait that will let us more easily reuse
/// some functionality across graphs. The topo_sort and
/// invalidate sort order are required, as all of our 
/// eventual audio graphs will use this functionality. 
pub trait Graph<N> {
    type Node;
    type Nid;
    type Connections;

     fn with_capacity(capacity: usize) -> Self;
     fn add_node(&mut self, node:N) -> Self::Nid;
     fn add_edge(&mut self, from: Self::Nid, to: Self::Nid);
     fn add_edges(&mut self, edges: &[(Self::Nid, Self::Nid)]);
     fn topo_sort(&self) -> Result<Vec<usize>, GraphError>;
     fn get_node_mut(&mut self, index: usize) -> &mut N;
     fn get_incoming(&self, index: usize) -> &Self::Connections;
}

/// A resizble graph for std environments, preferable for applications
/// with changing graph sizes. These will cause heap allocations which
/// can cause artifacts or missed frames in the audio thread, so it is 
/// better to preallocate when possible.
#[cfg(feature = "std")]
pub struct DynamicGraph<N> {
    pub nodes: Vec<N>,
    pub incoming: Vec<IndexSet<usize>>,
    pub outgoing: Vec<IndexSet<usize>>,
}
impl<N> Graph<N> for DynamicGraph<N> {
    type Nid = usize;
    type Node = N;
    type Connections = IndexSet<Self::Nid>;

    fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
            incoming: vec![IndexSet::with_capacity(capacity); capacity],
            outgoing: vec![IndexSet::with_capacity(capacity); capacity],
        }
    }
    #[inline(always)]
    fn get_node_mut(&mut self, index: usize) -> &mut N {
        &mut self.nodes[index]
    }

    #[inline(always)]
    fn get_incoming(&self, index: usize) -> &IndexSet<usize> {
        &self.incoming[index]
    }

    fn add_node(&mut self, node:N) -> Self::Nid {
        let id = self.nodes.len();
        self.nodes.push(node);
        id
    }
    fn add_edge(&mut self, from: usize, to: usize) {
        if (from == to) { return };
        self.outgoing[from].insert(to);
        self.incoming[to].insert(from);
    }
    fn add_edges(&mut self, edges: &[(usize, usize)]) {
        for (from, to) in edges {
            if from == to {
                continue
            }
            self.outgoing[*from].insert(*to);
            self.incoming[*to].insert(*from);
        }
    }
    /// TODO: can we use some scratch buffers to remove a runtime heap alloc?
    fn topo_sort(&self) -> Result<Vec<usize>, GraphError> {
        let mut indegree = vec![0; self.nodes.len()];

        for targets in &self.outgoing {
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
            if let Some(connections) = self.outgoing.get(node_index){
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
}

// TODO: Fixed Size Graph for embedded environments

// pub struct FixedGraph<const C: usize, N> {
//     nodes: [N; C],
//     incoming: [[usize; C]; C],
//     outgoing: [[usize; C]; C],
//     node_count: usize,
// }

// impl<const C: usize, N: Default> Graph<N> for FixedGraph<C, N> {
//     type Nid = usize;
//     type Node = N;
//     type Connections = [usize; C];

//     fn with_capacity(_capacity: usize) -> Self {
//         Self {
//             nodes: array::from_fn(|_| N::default()),
//             incoming: [[0; C]; C],
//             outgoing: [[0; C]; C],
//             node_count: 0,
//         }
//     }

//     fn add_node(&mut self, node: N) -> Self::Nid {
//         if self.node_count >= C {
//             panic!("cannot add node: reached fixed capacity {}", C);
//         }
//         let id = self.node_count;
//         self.nodes[id] = node;
//         self.node_count += 1;
//         id
//     }

//     fn add_edge(&mut self, from: usize, to: usize) {
//         if from == to { return; }
//         // only allow within the range of added nodes
//         if from < self.node_count && to < self.node_count {
//             self.outgoing[from][to] = 1;
//             self.incoming[to][from] = 1;
//         } else {
//             panic!("add_edge: node index out of bounds ({}, {})", from, to);
//         }
//     }

//     fn add_edges(&mut self, edges: &[(usize, usize)]) {
//         for &(from, to) in edges {
//             if from == to { continue }
//             if from < self.node_count && to < self.node_count {
//                 self.outgoing[from][to] = 1;
//                 self.incoming[to][from] = 1;
//             } else {
//                 panic!("add_edges: node index out of bounds ({}, {})", from, to);
//             }
//         }
//     }

//     fn topo_sort(&self) -> Result<Vec<usize>, GraphError> {
//         let n = self.node_count;
//         let mut indegree = [0usize; C];

//         for u in 0..n {
//             for v in 0..n {
//                 if self.outgoing[u][v] != 0 {
//                     indegree[v] += 1;
//                 }
//             }
//         }

//         let mut q = VecDeque::new();
//         for i in 0..n {
//             if indegree[i] == 0 {
//                 q.push_back(i);
//             }
//         }

//         let mut sorted = Vec::with_capacity(n);
//         while let Some(u) = q.pop_front() {
//             sorted.push(u);
//             for v in 0..n {
//                 if self.outgoing[u][v] != 0 {
//                     indegree[v] -= 1;
//                     if indegree[v] == 0 {
//                         q.push_back(v);
//                     }
//                 }
//             }
//         }

//         if sorted.len() == n {
//             Ok(sorted)
//         } else {
//             Err(GraphError::CycleDetected)
//         }
//     }

//     #[inline(always)]
//     fn get_node_mut(&mut self, index: usize) -> &mut N {
//         &mut self.nodes[index]
//     }

//     #[inline(always)]
//     fn get_incoming(&self, index: usize) -> &Self::Connections {
//         &self.incoming[index]
//     }
// }