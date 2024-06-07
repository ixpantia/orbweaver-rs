use std::{collections::HashMap, hash::BuildHasher};

/// Gets the equivalent values in `rel2` to a set in
/// `rel1`.
#[inline]
pub(crate) fn get_values_on_rel_map<H: BuildHasher>(
    ids: &[u32],
    map: &HashMap<u32, Vec<u32>, H>,
    out: &mut Vec<u32>,
) {
    ids.iter().for_each(|id| {
        if let Some(values) = map.get(id) {
            values.iter().for_each(|val| out.push(*val))
        }
    })
}
