use crate::{Error, RawObjectPointer, Result};
use std::mem::transmute;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct Immediate(pub i64);

impl Immediate {
    const SMALL_INTEGER_TAG: i64 = 1;
    const NUMBER_TAG: i64 = 3;

    pub fn new_i64(value: i64) -> Self {
        let unsigned_value: u64 = unsafe { transmute(value) };
        Self::new_u64(unsigned_value)
    }

    pub fn new_u64(value: u64) -> Self {
        Self(unsafe { transmute((value << Self::NUMBER_TAG) + 1) })
    }

    pub fn is_small_integer(&self) -> bool {
        self.0 & Self::SMALL_INTEGER_TAG != 0
    }

    pub fn as_integer(&self) -> Option<i64> {
        if self.is_small_integer() {
            if self.0 >> 63 == 1 {
                // negative
                let value =
                    (self.0 >> Self::NUMBER_TAG & 0x1FFFFFFFFFFFFFFF) - 0x1FFFFFFFFFFFFFFF - 1;
                Some(value)
            } else {
                // positive
                Some(self.0 >> Self::NUMBER_TAG)
            }
        } else {
            None
        }
    }

    pub fn try_as_integer(&self) -> Result<i64> {
        self.as_integer()
            .ok_or_else(|| Error::NotAnInteger(self.clone()))
    }
}

impl From<u64> for Immediate {
    fn from(value: u64) -> Self {
        Self::new_u64(value)
    }
}

impl From<i64> for Immediate {
    fn from(value: i64) -> Self {
        Self::new_i64(value)
    }
}

impl From<usize> for Immediate {
    fn from(value: usize) -> Self {
        Self::new_u64(value as u64)
    }
}

impl TryFrom<RawObjectPointer> for Immediate {
    type Error = Error;

    fn try_from(value: RawObjectPointer) -> Result<Self> {
        if value.is_immediate() {
            Ok(Self(value.as_i64()))
        } else {
            Err(Error::NotAnObject(value))
        }
    }
}
