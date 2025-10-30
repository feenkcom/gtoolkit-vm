use crate::assign_field;
use crate::objects::{Array, ArrayRef, Association, AssociationRef};
use num_traits::Zero;
use std::ops::Deref;
use vm_bindings::{ObjectPointer, Smalltalk};
use vm_object_model::{AnyObjectRef, Immediate, Object, ObjectRef, RawObjectPointer};

#[derive(Debug, PharoObject)]
#[repr(C)]
pub struct IdentityDictionary {
    this: Object,
    tally: Immediate,
    array: ArrayRef,
    association_class: ObjectRef,
}

impl IdentityDictionary {
    pub fn get_or_insert(
        &mut self,
        key: impl Into<AnyObjectRef>,
        default_value: impl FnOnce() -> AnyObjectRef,
    ) -> AnyObjectRef {
        let key = key.into();
        let index = self.scan_for(key).unwrap_or_else(|| {
            panic!(
                "Dictionary must have empty space; has {:?}. tally is {}. key is {:?}",
                self.array.len(),
                self.tally(),
                key
            )
        });

        let association = self.association_at(index);
        match association {
            None => {
                let default = default_value();
                self.at_new_index_put(index, key, default);
                default
            }
            Some(association) => association.value(),
        }
    }

    fn association_at(&self, index: usize) -> Option<AssociationRef> {
        let association_or_nil = self.array[index];
        if association_or_nil
            .as_object()
            .unwrap()
            .amount_of_slots()
            .is_zero()
        {
            None
        } else {
            Some(association_or_nil.try_into().unwrap())
        }
    }

    fn at_new_index_put(&mut self, index: usize, key: AnyObjectRef, value: AnyObjectRef) {
        let mut association = Association::new(self.association_class).unwrap();

        association.set_key(key);
        association.set_value(value);

        self.array.insert(index, association);
        self.tally = Immediate::new_i64(self.tally() + 1);
        self.full_check();
    }

    fn scan_for(&self, key: AnyObjectRef) -> Option<usize> {
        if key.is_immediate() {
            panic!("Immediate objects are not supported yet");
        }

        let hash = Smalltalk::identity_hash(ObjectPointer::from(key.as_i64()));
        let finish = self.array.len();
        let start = hash.rem_euclid(finish as u64) as usize;

        self.find_item_or_empty_slot(start, finish, key)
    }

    fn tally(&self) -> i64 {
        self.tally.as_integer().unwrap()
    }

    fn find_item_or_empty_slot(
        &self,
        start: usize,
        finish: usize,
        object: AnyObjectRef,
    ) -> Option<usize> {
        let nil_object = AnyObjectRef::from(RawObjectPointer::from(
            Smalltalk::primitive_nil_object().as_i64(),
        ))
        .as_object()
        .unwrap();

        for (index, association) in self.array.as_slice()[start..finish]
            .iter()
            .map(|each| each.as_object().unwrap())
            .enumerate()
        {
            let index = index + start;
            if association.equals(&nil_object).unwrap() {
                return Some(index);
            }

            if association.is_forwarded() {
                panic!(
                    "Association is forwarded: {:?} {:?}",
                    association,
                    association.header()
                );
            }

            let association = AssociationRef::try_from(AnyObjectRef::from(association)).unwrap_or_else(|error| {
                panic!(
                    "[Forward] Failed to convert an object to association {:?} {:?}. Error: {}. nil object: {:?}",
                    association,
                    association.deref(),
                    error,
                    nil_object
                )});
            if association.key().equals(&object).unwrap_or_else(|error| {
                panic!(
                    "[Forward] Failed to compare association key {:?} with object {:?}. Error: {}",
                    association.key(),
                    object,
                    error
                );
            }) {
                return Some(index);
            }
        }

        for (index, association) in self.array.as_slice()[0..start]
            .iter()
            .map(|each| each.as_object().unwrap())
            .enumerate()
        {
            if association.equals(&nil_object).unwrap() {
                return Some(index);
            }

            let association = AssociationRef::try_from(AnyObjectRef::from(association)).unwrap();
            if association
                .key()
                .equals(&object)
                .expect("[Backward] Compare association key with object")
            {
                return Some(index);
            }
        }
        None
    }

    fn full_check(&mut self) {
        let available_space = self.array.len() - (self.tally() as usize);
        if available_space < (self.array.len() / 4).max(1) {
            self.grow();
        }
    }

    fn scan_all_for(&self, key: AnyObjectRef) -> Vec<usize> {
        let nil_object = AnyObjectRef::from(RawObjectPointer::from(
            Smalltalk::primitive_nil_object().as_i64(),
        ))
        .as_object()
        .unwrap();

        let mut indices = vec![];

        for (index, each) in self
            .array
            .iter()
            .map(|each| each.as_object().unwrap())
            .enumerate()
        {
            if !each.equals(&nil_object).unwrap() {
                let association = AssociationRef::try_from(AnyObjectRef::from(each)).unwrap();
                if association.key().equals(&key).unwrap() {
                    indices.push(index);
                }
            }
        }
        indices
    }

    fn grow(&mut self) {
        let old_elements = self.array;

        let new_array_len = old_elements.len() * 2;
        let new_array = Array::new(new_array_len).unwrap();
        assign_field!(self.this, self.array, new_array);

        if new_array.len() != new_array_len {
            panic!(
                "Failed to allocate an array of the requested size; expected: {}, actual: {}",
                self.array.len(),
                new_array_len
            );
        }

        self.tally = Immediate::new_i64(0);

        let nil_object = AnyObjectRef::from(RawObjectPointer::from(
            Smalltalk::primitive_nil_object().as_i64(),
        ));
        for each in old_elements.iter() {
            if !each.equals(&nil_object).unwrap() {
                self.no_check_add(each.clone().try_into().unwrap());
            }
        }
    }

    fn no_check_add(&mut self, association: AssociationRef) {
        let key = association.key();
        let index = self.scan_for(key).expect("Find an available slot");
        self.array.insert(index, association);
        self.tally = Immediate::new_i64(self.tally() + 1);
    }
}
