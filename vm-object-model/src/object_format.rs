#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u8)]
pub enum ObjectFormat {
    /// 0 sized objects (UndefinedObject True False et al)
    ZeroSized = 0,
    /// non-indexable objects with inst vars (Point et al)
    NonIndexable = 1,
    /// indexable objects with no inst vars (Array et al)
    IndexableWithoutInstVars = 2,
    /// indexable objects with inst vars (MethodContext AdditionalMethodState et al)
    IndexableWithInstVars = 3,
    /// weak indexable objects with inst vars (WeakArray et al)
    WeakIndexable = 4,
    /// weak non-indexable objects with inst vars (ephemerons) (Ephemeron)
    WeakNonIndexable = 5,
    /// Forwarded Object, 1st field is pointer, rest of fields are ignored
    Forwarded = 7,
    /// 64-bit indexable
    Indexable64 = 9,
    /// 10 - 11 32-bit indexable
    Indexable32(u8),
    /// 12 - 15 16-bit indexable
    Indexable16(u8),
    /// 16 - 23 8-bit indexable
    Indexable8(u8),
    /// 24 - 31 compiled method
    CompiledMethod(u8),
    Unsupported(u8),
}

impl ObjectFormat {
    pub const fn into_bits(self) -> u8 {
        match self {
            Self::ZeroSized => 0,
            Self::NonIndexable => 1,
            Self::IndexableWithoutInstVars => 2,
            Self::IndexableWithInstVars => 3,
            Self::WeakIndexable => 4,
            Self::WeakNonIndexable => 5,
            Self::Forwarded => 7,
            Self::Indexable64 => 9,
            Self::Indexable32(value) => value,
            Self::Indexable16(value) => value,
            Self::Indexable8(value) => value,
            Self::CompiledMethod(value) => value,
            Self::Unsupported(value) => value,
        }
    }
    pub const fn from_bits(value: u8) -> Self {
        match value {
            0 => Self::ZeroSized,
            1 => Self::NonIndexable,
            2 => Self::IndexableWithoutInstVars,
            3 => Self::IndexableWithInstVars,
            4 => Self::WeakIndexable,
            5 => Self::WeakNonIndexable,
            7 => Self::Forwarded,
            9 => Self::Indexable64,
            10..=11 => Self::Indexable32(value),
            12..=15 => Self::Indexable16(value),
            16..=23 => Self::Indexable8(value),
            24..=31 => Self::CompiledMethod(value),
            _ => Self::Unsupported(value),
        }
    }

    pub fn amount_of_indexable_units(&self, amount_of_slots: usize) -> usize {
        const SHIFT_FOR_WORD: u8 = 3;
        match self {
            ObjectFormat::ZeroSized => amount_of_slots,
            ObjectFormat::NonIndexable => amount_of_slots,
            ObjectFormat::IndexableWithoutInstVars => amount_of_slots,
            ObjectFormat::IndexableWithInstVars => amount_of_slots,
            ObjectFormat::WeakIndexable => amount_of_slots,
            ObjectFormat::WeakNonIndexable => amount_of_slots,
            ObjectFormat::Forwarded => 0,
            ObjectFormat::Indexable64 => amount_of_slots,
            ObjectFormat::Indexable32(format) => {
                (amount_of_slots << (SHIFT_FOR_WORD - 2)) - (format & 1) as usize
            }
            ObjectFormat::Indexable16(format) => {
                (amount_of_slots << (SHIFT_FOR_WORD - 1)) - (format & 3) as usize
            }
            ObjectFormat::Indexable8(format) => {
                (amount_of_slots << SHIFT_FOR_WORD) - (format & 7) as usize
            }
            ObjectFormat::CompiledMethod(format) => {
                (amount_of_slots << SHIFT_FOR_WORD) - (format & 7) as usize
            }
            ObjectFormat::Unsupported(_) => 0,
        }
    }
}
