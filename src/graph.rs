use std::collections::VecDeque;
pub struct Graph<T> {
    nodes: Vec<GraphNode<T>>,
    edges: Vec<Edge>,
}

pub type NodeIndex = usize;

pub struct GraphNode<T> {
    data: T,
    first_outgoing_edge: Option<EdgeIndex>,
}

pub type EdgeIndex = usize;

pub struct Edge {
    target: NodeIndex,
    next_outgoing_edge: Option<EdgeIndex>
}

impl<T> Graph<T> {
    pub fn new(capacity: usize) -> Self {
        Graph { nodes: Vec::with_capacity(capacity),
                edges: Vec::with_capacity(capacity), }
    }

    pub fn add_node(&mut self, data: T) -> NodeIndex {
        let index = self.nodes.len();
        self.nodes.push(GraphNode { data, first_outgoing_edge: None });
        index
    }

    pub fn add_edge(&mut self, source: NodeIndex, target: NodeIndex) {
        let edge_index = self.edges.len();
        let node_data = &mut self.nodes[source];
        self.edges.push(Edge {
            target: target,
            next_outgoing_edge: node_data.first_outgoing_edge
        });
        node_data.first_outgoing_edge = Some(edge_index);
    }

    fn successors(&self, source: NodeIndex) -> Successors<T> {
        let first_outgoing_edge = self.nodes[source].first_outgoing_edge;
        Successors { graph: self, current_edge_index: first_outgoing_edge }
    }

    #[inline(always)]
    pub fn get_node_inner_ref(&mut self, index: usize) -> &mut T{
        &mut self.nodes[index].data
    }

    pub fn topo_sort(&self) -> Option<Vec<NodeIndex>> {
        let mut indegree = vec![0; self.nodes.len()];

        for node in &self.edges {
            indegree[node.target] += 1;
        }

        let mut no_incoming_edges_queue = VecDeque::new();
        for (i, count) in indegree.iter().enumerate() {
            if *count == 0 {
                no_incoming_edges_queue.push_back(i);
            }
        }

        let mut sorted = Vec::with_capacity(self.nodes.len());
        while let Some(nx) = no_incoming_edges_queue.pop_front() {
            sorted.push(nx);
            for t in self.successors(nx){
                indegree[t] -= 1;
                if indegree[t] == 0 {
                    no_incoming_edges_queue.push_back(t);
                }
            }
        }

        if sorted.len() == self.nodes.len(){
            Some(sorted)
        }
        // Here, we don't have a directed graph
        else {
            println!("topo sort failed");
            for x in sorted {
                println!("{:?}", x);
            }
            None
        }
    }
}

pub struct Successors<'a, T> {
    graph: &'a Graph<T>,
    current_edge_index: Option<EdgeIndex>,
}

impl<'a, T> Iterator for Successors<'a, T> {
    type Item = NodeIndex;

    fn next(&mut self) -> Option<NodeIndex> {
        match self.current_edge_index {
            None => None,
            Some(edge_num) => {
                let edge = &self.graph.edges[edge_num];
                self.current_edge_index = edge.next_outgoing_edge;
                Some(edge.target)
            }
        }
    }
}


mod test {
    use super::*;

    #[test]
    fn example() {

        let mut graph_one = Graph::new(4);

        let na0 = graph_one.add_node(0);
        let na1 = graph_one.add_node(1);
        let na2 = graph_one.add_node(2);
        let na3 = graph_one.add_node(3);

        graph_one.add_edge(na3, na0);
        graph_one.add_edge(na1, na0);
        graph_one.add_edge(na2, na0);

        let topo_one = graph_one.topo_sort();

        assert_eq!(topo_one, Some(vec![1,2,3,0]));


        let mut graph_two = Graph::new(6);

        let nb0 = graph_two.add_node(0);
        let nb1 = graph_two.add_node(1);
        let nb2 = graph_two.add_node(2);
        let nb3 = graph_two.add_node(3);
        let nb4 = graph_two.add_node(4);
        let nb5 = graph_two.add_node(5);
        let nb6 = graph_two.add_node(6);

        graph_two.add_edge(nb0, nb1);
        graph_two.add_edge(nb0, nb2);
        graph_two.add_edge(nb1, nb2);
        graph_two.add_edge(nb2, nb3);
        graph_two.add_edge(nb3, nb4);
        graph_two.add_edge(nb4, nb5);
        graph_two.add_edge(nb5, nb6);

        let topo_sort_two = graph_two.topo_sort();


        // assert_eq!(topo_sort, None);
        assert_eq!(topo_sort_two, Some(vec![0,1,2,3,4,5,6]))
    }
}