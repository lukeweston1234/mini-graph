use indexmap::IndexSet;
use std::collections::VecDeque;

#[derive(Debug)]
pub enum GraphError {
    InvalidNode(usize),
    CycleDetected
}

pub struct Graph<N> {
    nodes: Vec<N>,
    edges: Vec<IndexSet<usize>>
}

impl<N> Graph<N>{
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
            edges: vec![IndexSet::default(); capacity]
        }
    }
    pub fn add_node(&mut self, data: N) -> usize{
        let id = self.nodes.len();
        self.nodes.push(data);
        id
    }
    pub fn add_edge(&mut self, source: usize, target: usize) -> Result<(), GraphError> {
        if source >= self.edges.len(){
            return Err(GraphError::InvalidNode(source));
        }
        if target >= self.edges.len(){
            return Err(GraphError::InvalidNode(target));
        }
        self.edges[source].insert(target);
        Ok(())
    }
    pub fn topo_sort(&self) -> Result<Vec<usize>, GraphError> {
        let mut indegree = vec![0; self.nodes.len()];

        for targets in &self.edges {
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
            if let Some(connections) = self.edges.get(node_index){
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    #[test]
    fn test_add_node() {
        let mut g: Graph<&'static str> = Graph::with_capacity(8);
        let id = g.add_node("node0");
        assert_eq!(id, 0);
        assert_eq!(g.nodes.len(), 1);
        assert!(g.edges[0].is_empty());
    }

    #[test]
    fn test_topo_sort_simple_chain() {
        let mut g: Graph<&str> =  Graph::with_capacity(8);
        let a = g.add_node("A");
        let b = g.add_node("B");
        let c = g.add_node("C");
        let _ = g.add_edge(a, b);
        let _ = g.add_edge(b, c);

        let sorted = g.topo_sort().expect("should be a DAG");
        assert_eq!(sorted, vec![a, b, c]);
    }

    #[test]
    fn test_topo_sort_multiple_roots() {
        let mut g: Graph<&str> =  Graph::with_capacity(8);
        let x = g.add_node("X");
        let y = g.add_node("Y");
        let z = g.add_node("Z");
        // X→Z, Y→Z
        let _ = g.add_edge(x, z);
        let _ = g.add_edge(y, z);

        let sorted = g.topo_sort().expect("should be a DAG");
        assert_eq!(sorted.last(), Some(&z));
        assert_eq!(sorted, vec![x, y, z]);
    }

    #[test]
    fn test_topo_sort_cycle_detection() {
        let mut g: Graph<&str> = Graph::with_capacity(8);
        let p = g.add_node("P");
        let q = g.add_node("Q");
        let _ = g.add_edge(p, q);
        let _ = g.add_edge(q, p);

        assert!(g.topo_sort().is_err(), "should detect a cycle");
    }
}