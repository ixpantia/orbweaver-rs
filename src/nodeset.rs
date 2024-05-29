use rand::{Rng, RngCore};
use std::num::NonZeroI32;

use serde::{Deserialize, Serialize};

use crate::{
    prelude::{DuplicateNode, GraphInteractionResult},
    NodeId,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeSet<Data> {
    ids: Vec<NodeId>,
    hrids: Vec<String>,
    data: Vec<Data>,
}

impl<Data> NodeSet<Data> {
    pub fn new() -> Self {
        NodeSet {
            ids: Vec::new(),
            hrids: Vec::new(),
            data: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }
    pub fn get_id(&self, hrid: &str) -> Option<NodeId> {
        self.hrids
            .iter()
            .position(|h| h == hrid)
            .map(|index| unsafe { *self.ids.get_unchecked(index) })
    }
    #[inline(always)]
    pub(crate) fn add_entry_unchecked(&mut self, hrid: &str, node_id: NodeId, data: Data) {
        self.hrids.push(hrid.to_owned());
        self.ids.push(node_id);
        self.data.push(data);
    }
    #[inline(always)]
    pub(crate) fn get_index(&self, node_id: NodeId) -> Option<usize> {
        self.ids.iter().position(|&id| id == node_id)
    }
    pub fn add_node(&mut self, hrid: &str, data: Data) -> Result<NodeId, DuplicateNode> {
        match self.get_id(hrid) {
            Some(_) => Err(DuplicateNode::new(hrid)),
            None => {
                self.hrids.push(hrid.to_string());
                let node_id = loop {
                    let node_id = NodeId(rand::thread_rng().next_u32() as i32);
                    match self.get_index(node_id) {
                        Some(_) => continue,
                        None => break node_id,
                    }
                };
                self.ids.push(node_id);
                self.data.push(data);
                Ok(node_id)
            }
        }
    }
    pub fn remove_node(&mut self, node_id: NodeId) {
        if let Some(index) = self.get_index(node_id) {
            self.hrids.swap_remove(index);
            self.ids.swap_remove(index);
            self.data.swap_remove(index);
        }
    }
    pub fn get_data_mut(&mut self, node_id: NodeId) -> Option<&mut Data> {
        self.get_index(node_id)
            .map(|i| unsafe { self.data.get_unchecked_mut(i) })
    }
    pub fn get_key_value(&self, node_id: NodeId) -> Option<(&str, &Data)> {
        self.get_index(node_id).map(|i| {
            let node_hrid = unsafe { self.hrids.get_unchecked(i) };
            let node_data = unsafe { self.data.get_unchecked(i) };
            (node_hrid.as_str(), node_data)
        })
    }
    pub fn node_ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.ids.iter().copied()
    }
    pub fn into_dataless(&self) -> NodeSet<()> {
        let mut dataless_vec: Vec<()> = Vec::with_capacity(self.ids.len());
        dataless_vec.resize(self.ids.len(), ());
        NodeSet {
            ids: self.ids.clone(),
            hrids: self.hrids.clone(),
            data: dataless_vec,
        }
    }
}

impl<Data> NodeSet<&Data>
where
    Data: Clone,
{
    pub fn cloned(self) -> NodeSet<Data> {
        NodeSet {
            ids: self.ids,
            data: self.data.into_iter().cloned().collect(),
            hrids: self.hrids,
        }
    }
}
