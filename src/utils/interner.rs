use std::collections::HashMap;

use fxhash::FxBuildHasher;

use super::sym::Sym;

#[derive(Clone)]
pub(crate) struct InternerBuilder {
    count: Sym,
    map_strs: HashMap<Box<str>, Sym>,
}

impl InternerBuilder {
    pub(crate) fn new() -> Self {
        InternerBuilder {
            count: Sym::new(0),
            map_strs: HashMap::new(),
        }
    }

    pub(crate) fn get_or_intern(&mut self, val: impl AsRef<str>) -> Sym {
        match self.map_strs.get(val.as_ref()) {
            Some(sym) => *sym,
            None => {
                let sym = *self
                    .map_strs
                    .entry(val.as_ref().into())
                    .or_insert(self.count);
                self.count += 1;
                sym
            }
        }
    }

    pub(crate) fn build(self) -> Resolver {
        let mut indices = Vec::new();
        let mut arena = Vec::new();
        for (key, i) in self.map_strs {
            let key_bytes = key.as_bytes();
            indices.push((i, arena.len(), key_bytes.len()));
            arena.extend_from_slice(key.as_bytes());
        }
        let arena: Box<[u8]> = Box::from(arena);
        let arena_ptr = arena.as_ptr();
        let mut strs = Vec::new();
        let mut strs_map = HashMap::default();
        indices.sort_by_key(|(i, _, _)| *i);
        for (i, start, end) in indices {
            let current_str = unsafe {
                std::str::from_utf8_unchecked(std::slice::from_raw_parts(arena_ptr.add(start), end))
            };
            strs_map.insert(current_str, i);
            strs.push(current_str);
        }
        let strs = Box::from(strs);
        strs_map.shrink_to_fit();
        Resolver {
            strs_map,
            strs,
            arena,
        }
    }
}

pub(crate) struct Resolver {
    // This isnt actually static btw. This implements
    // unsafe self referencing
    //
    // The 'static str points to bytes in the arena
    strs_map: HashMap<&'static str, Sym, FxBuildHasher>,
    strs: Box<[&'static str]>,
    #[allow(unused)]
    arena: Box<[u8]>,
}

impl Resolver {
    #[inline(always)]
    pub(crate) fn get(&self, val: &str) -> Option<Sym> {
        self.strs_map.get(val).copied()
    }
    #[inline(always)]
    pub(crate) unsafe fn resolve_unchecked(&self, sym: Sym) -> &str {
        self.strs.get_unchecked(sym.into_usize())
    }
    #[inline(always)]
    pub(crate) unsafe fn resolve_many_unchecked_from_slice(&self, syms: &[Sym]) -> Vec<&str> {
        syms.iter()
            .map(|sym| *self.strs.get_unchecked(sym.into_usize()))
            .collect()
    }
    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.strs.len()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Resolver {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(self.strs.len()))?;
        for val in self.strs.iter() {
            seq.serialize_element(val)?;
        }
        seq.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Resolver {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut builder = InternerBuilder::new();
        let values: Vec<Box<str>> = Vec::deserialize(deserializer)?;
        for val in values {
            match builder.map_strs.entry(val) {
                std::collections::hash_map::Entry::Vacant(vac) => {
                    vac.insert(builder.count);
                    builder.count += 1;
                }
                std::collections::hash_map::Entry::Occupied(_) => {
                    return Err(serde::de::Error::custom("Duplicate value"));
                }
            }
        }

        Ok(builder.build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_build_interner() {
        let mut builder = InternerBuilder::new();
        let int1 = builder.get_or_intern("Hello");
        let int2 = builder.get_or_intern("World");
        let resolver = builder.build();
        assert_eq!(resolver.strs, Box::from(["Hello", "World"]));
        assert_eq!(int1, 0);
        assert_eq!(int2, 1);
    }

    #[test]
    fn can_access_after_move() {
        let mut builder = InternerBuilder::new();
        let int1 = builder.get_or_intern("Hello");
        let int2 = builder.get_or_intern("World");
        let resolver = builder.build();
        assert_eq!(resolver.strs, Box::from(["Hello", "World"]));
        assert_eq!(int1, 0);
        assert_eq!(int2, 1);

        let resolver2 = resolver;

        assert_eq!(resolver2.strs, Box::from(["Hello", "World"]));
    }
}
