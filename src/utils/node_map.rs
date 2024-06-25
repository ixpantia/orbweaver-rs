use fxhash::FxHashSet;

use super::sym::Sym;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub(crate) enum LazySet {
    Initialized(FxHashSet<Sym>),
    Uninitialized,
    Empty,
}

impl LazySet {
    pub(crate) fn or_init(&mut self) -> &mut FxHashSet<Sym> {
        if let LazySet::Uninitialized = self {
            *self = LazySet::Initialized(FxHashSet::default());
        }

        if let LazySet::Initialized(ref mut hs) = self {
            hs
        } else {
            // This code is unreachable, we just initialized it
            unsafe { std::hint::unreachable_unchecked() }
        }
    }

    pub(crate) fn into_empty(&mut self) {
        *self = LazySet::Empty
    }

    /// Returns `true` if the lazy set is [`Initialized`].
    ///
    /// [`Initialized`]: LazySet::Initialized
    #[must_use]
    fn is_initialized(&self) -> bool {
        matches!(self, Self::Initialized(..))
    }

    /// Returns `true` if the lazy set is [`Empty`].
    ///
    /// [`Empty`]: LazySet::Empty
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Returns `true` if the lazy set is [`Uninitialized`].
    ///
    /// [`Uninitialized`]: LazySet::Uninitialized
    #[must_use]
    pub(crate) fn is_uninitialized(&self) -> bool {
        matches!(self, Self::Uninitialized)
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub(crate) struct NodeMap {
    map: Vec<LazySet>,
}

impl NodeMap {
    pub(crate) fn new(n_nodes: usize) -> Self {
        let mut map = Vec::new();
        map.resize(n_nodes, LazySet::Uninitialized);
        Self { map }
    }
    #[inline]
    pub(crate) fn get(&self, node: Sym) -> &LazySet {
        &self.map[node.into_usize()]
    }
    #[inline]
    pub(crate) fn get_mut(&mut self, node: Sym) -> &mut LazySet {
        &mut self.map[node.into_usize()]
    }
    pub(crate) fn contains_key(&self, key: Sym) -> bool {
        self.map
            .get(key.into_usize())
            .map(|ls| ls.is_initialized())
            .unwrap_or(false)
    }
    pub(crate) fn iter(&self) -> impl Iterator<Item = (Sym, &LazySet)> {
        self.map
            .iter()
            .enumerate()
            .map(|(i, set)| (Sym::new(i as u32), set))
    }
    pub(crate) fn len(&self) -> usize {
        self.map.len()
    }
    pub(crate) fn initialized_keys(&self) -> Vec<Sym> {
        (0..self.len())
            .filter(|i| self.map[*i].is_initialized())
            .map(|i| Sym::new(i as u32))
            .collect()
    }
}
