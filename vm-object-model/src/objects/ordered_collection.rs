use crate::{AnyObject, AnyObjectMut, Immediate, RawObjectPointer};

#[derive(Debug)]
pub struct OrderedCollectionMut<'image> {
    object: AnyObjectMut<'image>,
    // first_index: Immediate,
    // last_index: Immediate
}

impl<'image> OrderedCollectionMut<'image> {
    pub fn add_last(&'image mut self, object: &AnyObject) {
        let this = self.object.as_object_unchecked_mut();

        let last_index_var = this
            .inst_var_at(2)
            .unwrap()
            .as_immediate_unchecked()
            .as_integer()
            .unwrap();

        let mut array_inst_var = this.inst_var_at_mut(0).unwrap();
        let mut array = array_inst_var
            .as_object_unchecked_mut()
            .try_as_array_mut()
            .unwrap();
        array.insert(last_index_var as usize, &object);

        let new_last_index_var = AnyObject::Immediate(Immediate::new_integer(last_index_var + 1));

        this.inst_var_at_put(2, &new_last_index_var);
    }
}

#[derive(Debug, Copy, Clone)]
pub struct OrderedCollectionPointer(RawObjectPointer);

impl OrderedCollectionPointer {
    pub fn as_raw_object_header(&self) -> RawObjectPointer {
        self.0
    }

    pub fn as_ordered_collection_mut(&mut self) -> OrderedCollectionMut {
        let object = self.0.reify_mut();

        OrderedCollectionMut { object }
    }
}

impl TryFrom<RawObjectPointer> for OrderedCollectionPointer {
    type Error = String;

    fn try_from(pointer: RawObjectPointer) -> Result<Self, Self::Error> {
        let object = pointer.reify();
        if let Some(object) = object.try_as_object() {
            if object.amount_of_slots() != 3 {
                return Err(format!(
                    "OrderedCollection must have three slots, but has {}.",
                    object.amount_of_slots()
                ));
            }

            Ok(Self(pointer))
        } else {
            Err("RawPointer is not an object".into())
        }
    }
}

impl<'image> TryFrom<AnyObjectMut<'image>> for OrderedCollectionMut<'image> {
    type Error = String;

    fn try_from(mut object: AnyObjectMut<'image>) -> Result<Self, Self::Error> {
        if !object.is_object() {
            return Err("OrderedCollection must be an object".into());
        }

        if object.amount_of_slots() != 3 {
            return Err(format!(
                "OrderedCollection must have three slots, but has {}.",
                object.amount_of_slots()
            ));
        }

        Ok(Self { object })
    }
}
