use crate::objects::{Array, ArrayRef, Association, AssociationRef};
use num_traits::Zero;
use std::ops::{Deref, DerefMut};
use vm_bindings::{ObjectPointer, Smalltalk};
use vm_object_model::{
    AnyObjectRef, Error, Immediate, Object, ObjectRef, RawObjectPointer, Result,
};

#[derive(Debug)]
#[repr(C)]
pub struct IdentityDictionary {
    this: Object,
    tally: Immediate,
    array: ArrayRef,
    association_class: ObjectRef,
}

impl IdentityDictionary {
    pub fn insert(&mut self, key: AnyObjectRef, value: AnyObjectRef) {
        let index = self.scan_for(key).unwrap();
        let mut association = self.array[index].as_object().unwrap();

        // we know that an inner array contains either nils or associations.
        // nil objects have zero slots, so use that
        if association.amount_of_slots().is_zero() {
            self.at_new_index_put(index, key, value);
        } else {
            association.inst_var_at_put(1, value);
        }
    }

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

        let mut association = self.association_at(index);
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
        self.tally = Immediate::new_integer(self.tally() + 1);
        self.full_check();
    }

    fn scan_for(&self, key: AnyObjectRef) -> Option<usize> {
        if key.is_immediate() {
            panic!("Immediate objects are not supported yet");
        }

        let hash = Smalltalk::identity_hash(ObjectPointer::from(key.as_i64()));
        let finish = self.array.len();
        let start = hash.rem_euclid(finish as u32) as usize;

        println!(
            "scan for: {:?}; hash: {:?}; start: {}; finish: {}",
            key, hash, start, finish
        );

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
        let nil_object = AnyObjectRef::from(RawObjectPointer::from(Smalltalk::nil_object().as_i64())).as_object().unwrap();

        for (index, association) in self.array.as_slice()[start..finish]
            .iter()
            .map(|each| each.as_object().unwrap())
            .enumerate()
        {
            let index = index + start;
            if association.equals(&nil_object).unwrap() {
                println!("[Forward] Found empty slot at {}", index);
                return Some(index);
            }

            if association.is_forwarded() {
                panic!("Association is forwarded: {:?} {:?}", association, association.header());
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
                error!(
                    "[Forward] Failed to compare association key {:?} with object {:?}. Error: {}",
                    association.key(),
                    object,
                    error
                );
                false
            }) {
                println!("[Forward] Found object at {}", index);
                return Some(index);
            }
        }

        for (index, association) in self.array.as_slice()[0..start]
            .iter()
            .map(|each| each.as_object().unwrap())
            .enumerate()
        {
            if association.equals(&nil_object).unwrap() {
                println!("[Backward] Found empty slot at {}", index);
                return Some(index);
            }

            let association = AssociationRef::try_from(AnyObjectRef::from(association)).unwrap();
            if association
                .key()
                .equals(&object)
                .expect("[Backward] Compare association key with object")
            {
                println!("[Backward] Found empty slot at {}", index);
                return Some(index);
            }
        }
        println!("Could not find any slot for {:?}", object);
        None
    }

    fn full_check(&mut self) {
        let available_space = self.array.len() - (self.tally() as usize);
        if available_space < (self.array.len() / 4).max(1) {
            self.grow();
        }
    }

    fn grow(&mut self) {
        error!("Growing identity dictionary");

        let old_elements = self.array;

        let new_array_len = old_elements.len() * 2;
        let new_array = Array::new(new_array_len).unwrap();

        if new_array.len() != new_array_len {
            panic!(
                "Failed to allocate an array of the requested size; expected: {}, actual: {}",
                self.array.len(),
                new_array_len
            );
        }

        self.tally = Immediate::new_integer(0);

        let nil_object =
            AnyObjectRef::from(RawObjectPointer::from(Smalltalk::nil_object().as_i64()));
        for each in old_elements.iter() {
            if !each.equals(&nil_object).unwrap() {
                self.no_check_add(new_array, each.clone().try_into().unwrap());
            }
        }

        self.array = new_array;
    }

    fn no_check_add(&mut self, mut array: ArrayRef, association: AssociationRef) {
        let key = association.key();
        let index = self.scan_for(key).expect("Find an available slot");
        array.insert(index, association);
        self.tally = Immediate::new_integer(self.tally() + 1);
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct IdentityDictionaryRef(ObjectRef);

impl Deref for IdentityDictionaryRef {
    type Target = IdentityDictionary;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.cast() }
    }
}

impl DerefMut for IdentityDictionaryRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.cast_mut() }
    }
}

impl TryFrom<AnyObjectRef> for IdentityDictionaryRef {
    type Error = Error;

    fn try_from(value: AnyObjectRef) -> Result<Self> {
        let object = value.as_object()?;

        if object.amount_of_slots() != 3 {
            return Err(Error::InvalidType("IdentityDictionary".to_string()));
        }

        Ok(Self(object))
    }
}
