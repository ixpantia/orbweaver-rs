use super::{DirectedGraph, LazySet};

const DEFAULT_MAX_PRINT_SIZE: usize = 15;

fn get_max_str_length(graph: &DirectedGraph) -> usize {
    let mut n_printed = 0;
    let mut max_string_length = DEFAULT_MAX_PRINT_SIZE;
    'outer: for (parent, children) in graph.children_map.iter() {
        if let LazySet::Initialized(children) = children {
            for &child in children.iter() {
                n_printed += 1;
                max_string_length = max_string_length
                    .max(graph.resolve(parent).len())
                    .max(graph.resolve(child).len());
                if n_printed == 10 {
                    break 'outer;
                }
            }
        }
    }
    max_string_length
}

impl std::fmt::Debug for DirectedGraph {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let n_nodes = self.children_map.len();
        let n_edges = self.n_edges;
        let n_roots = self.roots.len();
        let n_leaves = self.leaves.len();
        let max_string_length = get_max_str_length(self);
        writeln!(f, "# of nodes: {n_nodes}")?;
        writeln!(f, "# of edges: {n_edges}")?;
        writeln!(f, "# of roots: {n_roots}")?;
        writeln!(f, "# of leaves: {n_leaves}")?;
        writeln!(f)?;
        writeln!(
            f,
            "| {:^width$} | {:^width$} |",
            "Parent",
            "Child",
            width = max_string_length
        )?;
        writeln!(
            f,
            "| {:-<width$} | {:-<width$} |",
            "",
            "",
            width = max_string_length
        )?;
        let mut n_printed = 0;
        'outer: for (parent, children) in self.children_map.iter() {
            match children {
                LazySet::Initialized(children) => {
                    for &child in children.iter() {
                        n_printed += 1;
                        let parent = self.resolve(parent);
                        let child = self.resolve(child);
                        writeln!(
                            f,
                            "| {:width$.width$} | {:width$.width$} |",
                            parent,
                            child,
                            width = max_string_length
                        )?;
                        if n_printed == 10 {
                            break 'outer;
                        }
                    }
                }
                _ => continue,
            }
        }

        if n_nodes > 10 {
            writeln!(f, "Omitted {} nodes", n_nodes - 10)?;
        }

        Ok(())
    }
}

//#[cfg(test)]
//mod tests {
//    use crate::directed::DirectedGraphBuilder;
//
//    #[test]
//    fn test_debug_printing() {
//        let mut builder = DirectedGraphBuilder::new();
//        builder.add_path(["0", "111", "222", "333", "444", "4"]);
//        builder.add_path(["0", "999", "4"]);
//        builder.add_path(["0", "1", "2", "3", "4"]);
//        builder.add_path(["0", "4"]);
//        let graph = builder.build_acyclic().unwrap();
//
//        assert_eq!(
//            format!("{:?}", graph),
//            r#"# of nodes: 10
//# of edges: 12
//# of roots: 1
//# of leaves: 1
//
//|     Parent      |      Child      |
//| --------------- | --------------- |
//| 0               | 1               |
//| 0               | 111             |
//| 0               | 999             |
//| 0               | 4               |
//| 111             | 222             |
//| 222             | 333             |
//| 333             | 444             |
//| 444             | 4               |
//| 999             | 4               |
//| 1               | 2               |
//Omitted 1 nodes
//"#,
//        );
//    }
//
//    #[test]
//    fn test_debug_printing_longer_than_15() {
//        let mut builder = DirectedGraphBuilder::new();
//        builder.add_edge("AAAAAAAAAAAAAAAAAAAAA", "B");
//        builder.add_edge("C", "AAAAAAAAAAAAAAAAAAAAA");
//        let graph = builder.build_acyclic().unwrap();
//
//        panic!("{:?}", graph);
//
//        assert_eq!(
//            format!("{:?}", graph),
//            r#"# of nodes: 11
//# of edges: 12
//# of roots: 1
//# of leaves: 1
//
//|     Parent      |      Child      |
//| --------------- | --------------- |
//| 0               | 1               |
//| 0               | 111             |
//| 0               | 999             |
//| 0               | 4               |
//| 111             | 222             |
//| 222             | 333             |
//| 333             | 444             |
//| 444             | 4               |
//| 999             | 4               |
//| 1               | 2               |
//Omitted 1 nodes
//"#,
//        );
//    }
//}
