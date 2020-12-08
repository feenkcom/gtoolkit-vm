use crate::bindings::sqInt;

extern "C" {
    pub fn numSlotsOf(oop: sqInt) -> sqInt;
}
