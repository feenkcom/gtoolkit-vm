use vm_object_model::{AnyObjectRef, Error, Object, ObjectFormat, ObjectHeader};

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

impl<'image> TryFrom<&'image Object> for ByteSymbol<'image> {
    type Error = Error;

    fn try_from(object: &'image Object) -> Result<Self, Self::Error> {
        match object.object_format() {
            ObjectFormat::Indexable8(_) => {
                let length = object.amount_of_indexable_units();
                let slice_ptr =
                    unsafe { object.as_ptr().offset(size_of::<ObjectHeader>() as isize) }
                        as *const u8;

                let source_bytes = unsafe { std::slice::from_raw_parts(slice_ptr, length) };
                let source_str = unsafe { std::str::from_utf8_unchecked(source_bytes) };

                Ok(ByteSymbol::new(object.header(), source_str))
            }
            _ => Err(Error::InvalidType("ByteSymbol".to_string())),
        }
    }
}
