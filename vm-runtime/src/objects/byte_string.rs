use std::fmt::{Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::slice;
use vm_object_model::{AnyObjectRef, Error, Object, ObjectFormat, ObjectRef, Result};

#[derive(Debug)]
#[repr(C)]
pub struct ByteString {
    this: Object,
}

impl ByteString {
    pub fn bytes(&self) -> &[u8] {
        let len = self.amount_of_indexable_units();
        unsafe { slice::from_raw_parts(self.first_fixed_field_ptr() as _, len) }
    }

    pub fn as_str(&self) -> &str {
        str::from_utf8(self.bytes()).unwrap()
    }
}

impl Display for ByteString {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Deref for ByteString {
    type Target = Object;
    fn deref(&self) -> &Self::Target {
        &self.this
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct ByteStringRef(ObjectRef);

impl Deref for ByteStringRef {
    type Target = ByteString;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.cast() }
    }
}

impl DerefMut for ByteStringRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.cast_mut() }
    }
}

impl TryFrom<AnyObjectRef> for ByteStringRef {
    type Error = Error;

    fn try_from(value: AnyObjectRef) -> Result<Self> {
        let object = value.as_object()?;
        match object.object_format() {
            ObjectFormat::Indexable8(_) => Ok(Self(object)),
            _ => Err(Error::InvalidType("ByteSymbol".to_string())),
        }
    }
}

impl From<ByteStringRef> for AnyObjectRef {
    fn from(value: ByteStringRef) -> Self {
        value.0.into()
    }
}
