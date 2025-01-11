use crate::ObjectHeader;

#[derive(Debug, Clone)]
pub struct ByteSymbol<'obj> {
    header: &'obj ObjectHeader,
    string: &'obj str,
}

impl<'obj> ByteSymbol<'obj> {
    pub fn new(header: &'obj ObjectHeader, items: &'obj str) -> Self {
        Self {
            header,
            string: items,
        }
    }

    pub fn as_str(&self) -> &'obj str {
        self.string
    }

    pub fn hash(&self) -> u32 {
        hash_of(self.string)
    }
}

pub fn hash_of(string: &str) -> u32 {
    // todo get hash from ByteString class
    let mut hash: u32 = 13312;

    for char in string.chars() {
        hash = (hash + char as u32).wrapping_mul(1664525);
    }

    hash & 0xFFFFFFF
}
