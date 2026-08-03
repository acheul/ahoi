use super::*;

#[cfg(not(feature = "long-path"))]
const PATH_LENGTH: usize = 16usize;

#[cfg(feature = "long-path")]
const PATH_LENGTH: usize = 32usize;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct Path {
    length: usize,
    path: [u64; PATH_LENGTH],
}

impl Path {
    pub(crate) const fn new_empty() -> Self {
        Self {
            length: 0usize,
            path: [0u64; PATH_LENGTH],
        }
    }

    pub(crate) const fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub(crate) fn as_slice(&self) -> &[u64] {
        &self.path[..self.length]
    }

    pub(crate) fn push(&mut self, path_key: u64) {
        if self.length + 1 > PATH_LENGTH {
            panic!("Over derive limit")
        }
        self.path[self.length] = path_key;
        self.length += 1;

        return;
    }
}
