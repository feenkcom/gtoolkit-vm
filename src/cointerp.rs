use crate::bindings::sqInt;

extern "C" {
    pub fn numSlotsOf(oop: sqInt) -> sqInt;
}

extern "C" {
    pub fn marshallArgumentFromatIndexintoofTypewithSize(argumentsArrayOop: sqInt, i: sqInt , argHolder: sqInt, argType: sqInt, argTypeSize: sqInt);
}