use crate::ObjectFormat;
use bitfield_struct::bitfield;

/// Represents a Spur Object Header
/// | 8: numSlots		| (on a byte boundary)
/// | 2 bits			| (msb,lsb = {isMarked,?})
/// | 22: identityHash	| (on a word boundary)
/// | 3 bits			| (msb <-> lsb = {isGrey,isPinned,isRemembered}
/// | 5: format			| (on a byte boundary)
/// | 2 bits			| (msb,lsb = {isImmutable,?})
/// | 22: classIndex	| (on a word boundary) : LSB
#[bitfield(u64)]
#[derive(PartialEq, Eq)]
pub struct ObjectHeader {
    #[bits(22)]
    pub class_index: u32,
    #[bits(1)]
    _reserved_1: u8,
    pub is_immutable: bool,
    #[bits(5)]
    pub format: ObjectFormat,
    pub is_remembered: bool,
    pub is_pinned: bool,
    pub is_grey: bool,
    #[bits(22)]
    pub identity_hash: u32,
    #[bits(1)]
    _reserved_2: u8,
    pub is_marked: bool,
    pub num_slots: u8,
}

