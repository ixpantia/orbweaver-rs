trait NodeId: Eq + PartialEq + Clone + Ord + PartialOrd {}

#[derive(Clone)]
struct Node<ID: NodeId, Data> {
    id: ID,
    data: Data,
}

impl<ID: NodeId, Data> std::cmp::Eq for Node<ID, Data> {}

impl<ID: NodeId, Data> PartialEq for Node<ID, Data> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<ID: NodeId, Data> PartialOrd for Node<ID, Data> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl<ID: NodeId, Data> Ord for Node<ID, Data> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

#[derive(Clone)]
struct Edge<ID: NodeId> {
    from: ID,
    to: ID,
}

#[derive(Clone)]
struct DirectedGraph<ID: NodeId, Data> {
    nodes: Vec<Node<ID, Data>>,
    edges: Vec<Edge<ID>>,
}

impl<ID: NodeId, Data> DirectedGraph<ID, Data> {
    fn new() -> Self {
        DirectedGraph {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
    fn add_node(&mut self, id: ID, data: Data) -> Result<&mut Self, ()> {
        let new_node = Node { id, data };
        match self.nodes.binary_search(&new_node) {
            Ok(_) => Err(()),
            Err(index) => {
                self.nodes.insert(index, new_node);
                Ok(self)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
