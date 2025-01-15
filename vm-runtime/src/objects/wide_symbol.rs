use vm_object_model::{Error, Object, ObjectFormat, ObjectHeader};
use widestring::U32Str;

#[derive(Debug, Clone)]
pub struct WideSymbol<'obj> {
    header: &'obj ObjectHeader,
    string: &'obj U32Str,
}

impl<'obj> WideSymbol<'obj> {
    pub fn new(header: &'obj ObjectHeader, string: &'obj U32Str) -> Self {
        Self { header, string }
    }
}

impl<'image> TryFrom<&'image Object> for WideSymbol<'image> {
    type Error = Error;

    fn try_from(object: &'image Object) -> Result<Self, Self::Error> {
        match object.object_format() {
            ObjectFormat::Indexable32(_) => {
                let length = object.amount_of_indexable_units();
                let slice_ptr =
                    unsafe { object.as_ptr().offset(size_of::<ObjectHeader>() as isize) }
                        as *const u32;

                let source_str = unsafe { U32Str::from_ptr(slice_ptr, length) };

                Ok(WideSymbol::new(object.header(), source_str))
            }
            _ => Err(Error::InvalidType("WideSymbol".to_string())),
        }
    }
}
