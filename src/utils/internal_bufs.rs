use std::{cell::UnsafeCell, collections::VecDeque};

use fxhash::FxHashSet;

use super::sym::Sym;

#[derive(Default)]
pub(crate) struct InternalBufs {
    // Espacio en memoria para buffers
    pub(crate) u32x1_vec_0: UnsafeCell<Vec<Sym>>,
    pub(crate) u32x1_vec_1: UnsafeCell<Vec<Sym>>,
    pub(crate) u32x1_vec_2: UnsafeCell<Vec<Sym>>,
    pub(crate) u32x2_vec_0: UnsafeCell<Vec<(Sym, Sym)>>,
    pub(crate) u32x1_queue_0: UnsafeCell<VecDeque<Sym>>,
    pub(crate) u32x1_set_0: UnsafeCell<FxHashSet<Sym>>,
    pub(crate) usizex2_queue_0: UnsafeCell<VecDeque<(usize, usize)>>,
}
