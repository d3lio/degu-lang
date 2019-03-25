use std::borrow::Borrow;
use std::collections::HashSet;
use std::ffi::{CStr, CString};
use std::hash::Hash;

pub struct CStringInternPool {
    // TODO: could improve by crating a slab memory allocator for the strings
    pool: HashSet<CString>,
}

impl CStringInternPool {
    pub fn new() -> Self {
        Self {
            pool: HashSet::new(),
        }
    }

    pub fn intern<T>(&mut self, value: T) -> &CStr
    where
        T: Borrow<str> + PartialEq + Eq + Hash,
    {
        let name = CString::new(value.borrow()).unwrap();
        if !self.pool.contains(&name) {
            self.pool.insert(name.clone());
        }
        self.pool.get(&name).unwrap().as_c_str()
    }
}
