#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub(crate) struct Sym(u32);

impl std::hash::Hash for Sym {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl std::fmt::Debug for Sym {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Sym::RESERVED => write!(f, "RESERVED"),
            _ => write!(f, "{}", self.0),
        }
    }
}

impl PartialEq<u32> for Sym {
    fn eq(&self, other: &u32) -> bool {
        self.0.eq(other)
    }
}

impl std::ops::Add for Sym {
    type Output = Sym;
    fn add(self, rhs: Self) -> Self::Output {
        Sym(self.0.add(rhs.0))
    }
}

impl std::ops::Add<u32> for Sym {
    type Output = Sym;
    fn add(self, rhs: u32) -> Self::Output {
        Sym(self.0.add(rhs))
    }
}

impl std::ops::AddAssign for Sym {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl std::ops::AddAssign<u32> for Sym {
    fn add_assign(&mut self, rhs: u32) {
        self.0 += rhs;
    }
}

impl Sym {
    /// This value is reserved. It can be used
    /// for cases where we want to have
    /// a value that represents anything
    /// but a node.
    pub const RESERVED: Sym = Sym::new(u32::MAX);
    #[inline(always)]
    pub const fn new(v: u32) -> Self {
        Sym(v)
    }
    #[inline(always)]
    pub const fn is_reserved(self) -> bool {
        self.0 == u32::MAX
    }
    #[inline(always)]
    pub const fn into_usize(self) -> usize {
        self.0 as usize
    }
}
