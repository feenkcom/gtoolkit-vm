use crate::objects::{Array, ArrayRef};
use std::ops::{Deref, DerefMut};
use vm_bindings::Smalltalk;
use vm_object_model::{AnyObjectRef, Error, Immediate, Object, ObjectRef, Result};

#[derive(Debug)]
#[repr(C)]
pub struct OrderedCollection {
    this: Object,
    array: ArrayRef,
    first_index: Immediate,
    last_index: Immediate,
}

impl OrderedCollection {
    pub fn new(ordered_collection_class: ObjectRef) -> Result<OrderedCollectionRef> {
        Self::with_capacity(ordered_collection_class, 10)
    }

    pub fn with_capacity(
        ordered_collection_class: ObjectRef,
        capacity: usize,
    ) -> Result<OrderedCollectionRef> {
        let mut ordered_collection =
            Smalltalk::instantiate::<OrderedCollectionRef>(ordered_collection_class)?;
        ordered_collection.array = Array::new(capacity)?;
        ordered_collection.first_index = Immediate::new_integer(1);
        ordered_collection.last_index = Immediate::new_integer(0);
        Ok(ordered_collection)
    }

    pub fn add_last(&mut self, object: impl Into<AnyObjectRef>) {
        if self.last_index() == self.array.len() {
            println!("Time to grow!");
            self.make_room_at_last();
        }
        let last_index = self.last_index();

        println!("Add item to {:?}. Array: {:?}", self, self.array.deref());

        self.array.insert(last_index, object);
        self.last_index = Immediate::new_integer(last_index as i64 + 1);
    }

    pub fn len(&self) -> usize {
        self.last_index() - self.first_index() + 1
    }

    fn first_index(&self) -> usize {
        self.first_index.as_integer().unwrap() as usize
    }

    fn last_index(&self) -> usize {
        self.last_index.as_integer().unwrap() as usize
    }

    fn make_room_at_last(&mut self) {
        let tally = self.len();
        if (tally * 2) >= self.last_index() {
            return self.grow_at_last();
        }
    }

    //// makeRoomAtLast
    //     // 	"Make some empty slots at the end of the array. If we have more than 50% free space, then just move the elements, so that the last 50% of the slots are free, otherwise add new free slots to the end by growing. Precondition: lastIndex = array size"
    //     //
    //     // 	| tally newFirstIndex newLastIndex |
    //     // 	tally := self size.
    //     // 	tally * 2 >= lastIndex ifTrue: [ ^self growAtLast ].
    //     // 	tally = 0 ifTrue: [ ^self resetTo: 1 ].
    //     // 	newLastIndex := lastIndex // 2.
    //     // 	newFirstIndex := newLastIndex - lastIndex + firstIndex.
    //     // 	array
    //     // 		replaceFrom: newFirstIndex
    //     // 		to: newLastIndex
    //     // 		with: array
    //     // 		startingAt: firstIndex.
    //     // 	array from: newLastIndex + 1 to: lastIndex put: nil.
    //     // 	firstIndex := newFirstIndex.
    //     // 	lastIndex := newLastIndex

    fn grow_at_last(&mut self) {
        let mut new_array = Array::new((self.array.len() * 2).max(1)).unwrap();

        let start_index = self.first_index() - 1;
        let end_index = self.last_index();

        let target_slice = &mut new_array.as_slice_mut()[start_index..end_index];
        let source_slice = &self.array.as_slice()[start_index .. end_index];

        target_slice.copy_from_slice(source_slice);

        self.array = new_array;

        println!("Grew an internal array to {}", self.array.len());
    }
}

impl Deref for OrderedCollection {
    type Target = Object;

    fn deref(&self) -> &Self::Target {
        &self.this
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct OrderedCollectionRef(ObjectRef);

impl Deref for OrderedCollectionRef {
    type Target = OrderedCollection;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.cast() }
    }
}

impl DerefMut for OrderedCollectionRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.cast_mut() }
    }
}

impl TryFrom<AnyObjectRef> for OrderedCollectionRef {
    type Error = Error;

    fn try_from(value: AnyObjectRef) -> Result<Self> {
        const EXPECTED_AMOUNT_OF_SLOTS: usize = 3;
        let object = value.as_object()?;
        let actual_amount_of_slots = object.amount_of_slots();

        if actual_amount_of_slots != EXPECTED_AMOUNT_OF_SLOTS {
            return Err(Error::WrongAmountOfSlots {
                object_ref: object,
                expected: EXPECTED_AMOUNT_OF_SLOTS,
                actual: actual_amount_of_slots,
            }
            .into());
        }

        Ok(Self(object))
    }
}

impl From<OrderedCollectionRef> for AnyObjectRef {
    fn from(value: OrderedCollectionRef) -> Self {
        value.0.into()
    }
}
