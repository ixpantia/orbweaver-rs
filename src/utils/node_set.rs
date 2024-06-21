use std::rc::Rc;

#[derive(Clone)]
pub struct NodeVec {
    pub(crate) values: Vec<&'static str>,
    #[allow(dead_code)]
    pub(crate) arena: Rc<[u8]>,
}

impl PartialEq for NodeVec {
    fn eq(&self, other: &Self) -> bool {
        self.values.eq(&other.values)
    }
}

impl PartialEq<Vec<&str>> for NodeVec {
    fn eq(&self, other: &Vec<&str>) -> bool {
        self.values.eq(other)
    }
}

impl PartialEq<&[&str]> for NodeVec {
    fn eq(&self, other: &&[&str]) -> bool {
        self.values.eq(other)
    }
}

impl<const N: usize> PartialEq<[&str; N]> for NodeVec {
    fn eq(&self, other: &[&str; N]) -> bool {
        self.values.eq(other)
    }
}

const DEFAULT_MAX_PRINT_SIZE: usize = 25;

fn get_max_str_length(nodes: &[&str]) -> usize {
    let mut max_string_length = DEFAULT_MAX_PRINT_SIZE;
    for node in nodes.iter().take(10) {
        max_string_length = max_string_length.max(node.len());
    }
    max_string_length
}

impl std::fmt::Debug for NodeVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let n_nodes = self.values.len();
        let max_string_length = get_max_str_length(self.as_slice());
        writeln!(f, "# of nodes: {n_nodes}")?;
        writeln!(f, "| {:^width$} |", "Nodes", width = max_string_length)?;
        for node in self.as_slice().iter().take(10) {
            writeln!(f, "| {:^width$} |", node, width = max_string_length)?;
        }
        if n_nodes > 10 {
            writeln!(f, "| {:^width$} |", "...", width = max_string_length)?;
        }
        Ok(())
    }
}

impl NodeVec {
    #[inline]
    pub fn as_slice(&self) -> &[&str] {
        self.values.as_slice()
    }
    #[inline]
    pub fn as_vec(&self) -> Vec<&str> {
        self.values.clone()
    }
    #[inline]
    pub fn iter(&self) -> NodeVecIter<'_> {
        self.into_iter()
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&str> {
        self.values.get(index).copied()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a> IntoIterator for &'a NodeVec {
    type Item = &'a str;
    type IntoIter = NodeVecIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        NodeVecIter {
            node_set: self,
            i: 0,
        }
    }
}

pub struct NodeVecIter<'a> {
    node_set: &'a NodeVec,
    i: usize,
}

impl<'a> Iterator for NodeVecIter<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        let i = self.i;
        self.i += 1;
        self.node_set.values.get(i).copied()
    }
}
