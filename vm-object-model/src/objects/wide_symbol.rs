use crate::ObjectHeader;
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
