use vm_bindings::Smalltalk;
use vm_object_model::{ObjectRef, RawObjectPointer};

#[derive(Debug)]
pub struct OldMemorySpace {
    chunk: MemoryChunk,
}

impl OldMemorySpace {
    pub fn new() -> Self {
        Self {
            chunk: MemoryChunk::new(Smalltalk::old_space_start(), Smalltalk::old_space_end()),
        }
    }

    pub fn objects(&self) -> impl Iterator<Item = ObjectRef> {
        self.chunk.iter().filter(|each| each.is_enumerable())
    }
}

#[derive(Debug)]
pub struct EdenMemorySpace {
    chunk: MemoryChunk,
}

impl EdenMemorySpace {
    pub fn new() -> Self {
        Self {
            chunk: MemoryChunk::new(Smalltalk::eden_space_start(), Smalltalk::eden_space_end()),
        }
    }

    pub fn objects(&self) -> impl Iterator<Item = ObjectRef> {
        self.chunk.iter().filter(|each| each.is_enumerable())
    }
}

#[derive(Debug)]
pub struct PastMemorySpace {
    chunk: MemoryChunk,
}

impl PastMemorySpace {
    pub fn new() -> Self {
        Self {
            chunk: MemoryChunk::new(Smalltalk::past_space_start(), Smalltalk::past_space_end()),
        }
    }

    pub fn objects(&self) -> impl Iterator<Item = ObjectRef> {
        self.chunk.iter().filter(|each| each.is_enumerable())
    }
}

#[derive(Debug)]
struct MemoryChunk {
    start: RawObjectPointer,
    end: RawObjectPointer,
}

impl MemoryChunk {
    fn new(start: ObjectRef, end: RawObjectPointer) -> Self {
        Self {
            start: start.into_inner(),
            end,
        }
    }

    fn iter(&self) -> MemoryChunkIterator {
        let current = if self.start < self.end {
            Some(self.start)
        } else {
            None
        };

        MemoryChunkIterator {
            current,
            end: self.end,
        }
    }
}

#[derive(Debug)]
struct MemoryChunkIterator {
    current: Option<RawObjectPointer>,
    end: RawObjectPointer,
}

impl Iterator for MemoryChunkIterator {
    type Item = ObjectRef;

    fn next(&mut self) -> Option<Self::Item> {
        let current_ptr = self.current?;

        if current_ptr >= self.end {
            self.current = None;
            return None;
        }

        let object = unsafe { ObjectRef::from_raw_pointer_unchecked(current_ptr) };

        let next_ptr = Smalltalk::next_object(
            unsafe { ObjectRef::from_raw_pointer_unchecked(current_ptr) },
            self.end,
        )
        .into_inner();

        if next_ptr <= current_ptr || next_ptr >= self.end {
            self.current = None;
        } else {
            self.current = Some(next_ptr);
        }

        Some(object)
    }
}
