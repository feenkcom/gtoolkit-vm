use crate::{Immediate, ObjectHeader, ObjectRef};

#[repr(C)]
pub struct IdentityDictionary {
    header: ObjectHeader,
    tally: Immediate,
    array: ObjectRef
}

