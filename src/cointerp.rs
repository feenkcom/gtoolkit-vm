use crate::bindings::{sqInt, usqInt};
use libffi::low::ffi_type;

extern "C" {
    pub fn numSlotsOf(oop: sqInt) -> sqInt;
}

extern "C" {
    pub fn marshallArgumentFromatIndexintoofTypewithSize(
        argumentsArrayOop: sqInt,
        i: sqInt,
        argHolder: sqInt,
        argType: sqInt,
        argTypeSize: sqInt,
    );
}

extern "C" {
    pub fn marshallAndPushReturnValueFromofTypepoping(
        returnHolder: sqInt,
        ffiType: *mut ffi_type,
        argumentsAndReceiverCount: sqInt,
    );
}

extern "C" {
    pub fn instantiateClassindexableSizeisPinned(
        classObj: sqInt,
        nElements: usqInt,
        isPinned: sqInt,
    ) -> sqInt;
}
