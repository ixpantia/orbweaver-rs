use crate::utils::sym::Sym;

use super::{LazySet, NodeMap};

/// Gets the equivalent values in `rel2` to a set in
/// `rel1`.
#[inline]
pub(crate) fn get_values_on_rel_map(ids: &[Sym], map: &NodeMap, out: &mut Vec<Sym>) {
    ids.iter().for_each(|&id| {
        if let LazySet::Initialized(values) = map.get(id) {
            out.extend(values.iter().copied());
        }
    })
}
