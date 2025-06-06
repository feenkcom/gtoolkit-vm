use std::fmt::Debug;
use std::hash::Hash;
use std::ops::BitAnd;
use vm_bindings::{ObjectFieldIndex, ObjectPointer, Smalltalk};
use vm_object_model::{AnyObjectRef, Immediate, ObjectFormat, ObjectRef, RawObjectPointer};

impl GraphNode for AnyObjectRef {
    fn neighbors(&self) -> ObjectIterator {
        ObjectIterator::new(self.clone())
    }
}

pub trait GraphNode: Debug + Clone + Eq + Hash {
    fn neighbors(&self) -> ObjectIterator;
}

#[derive(Debug)]
pub struct ObjectIterator {
    pub object: AnyObjectRef,
    pub amount_of_fixed_fields: usize,
    pub amount_of_indexable_fields: usize,
    pub index: usize,
    pub is_context: bool,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ReferencedObject {
    InstanceVariable(AnyObjectRef, usize),
    ContextVariable(AnyObjectRef),
    ArrayItem(AnyObjectRef),
    Root(AnyObjectRef),
}

impl ReferencedObject {
    pub fn object(&self) -> AnyObjectRef {
        match *self {
            ReferencedObject::InstanceVariable(object, _) => object,
            ReferencedObject::ContextVariable(object) => object,
            ReferencedObject::ArrayItem(object) => object,
            ReferencedObject::Root(object) => object,
        }
    }
}

impl ObjectIterator {
    pub fn new(oop: AnyObjectRef) -> Self {
        if let Ok(object) = oop.as_object() {
            let amount_of_units = object.amount_of_indexable_units();
            let amount_of_fixed_fields = amount_of_fixed_fields(object, amount_of_units);

            if object.is_context() {
                return ObjectIterator {
                    object: oop,
                    amount_of_fixed_fields,
                    amount_of_indexable_fields: Smalltalk::context_size(object),
                    index: 0,
                    is_context: true,
                };
            }

            let amount_of_indexable_fields = amount_of_indexable_fields(object, amount_of_units);
            let is_ephemeron = object.header().format().is_ephemeron();
            let start_index = if is_ephemeron { 1 } else { 0 };

            ObjectIterator {
                object: oop,
                amount_of_fixed_fields,
                amount_of_indexable_fields,
                index: start_index,
                is_context: false,
            }
        } else {
            ObjectIterator {
                object: oop,
                amount_of_fixed_fields: 0,
                amount_of_indexable_fields: 0,
                index: 0,
                is_context: false,
            }
        }
    }
}

impl Iterator for ObjectIterator {
    type Item = ReferencedObject;
    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(object) = self.object.as_object() {
            loop {
                if self.index >= self.amount_of_fixed_fields {
                    break;
                }
                let next = if self.is_context {
                    ReferencedObject::ContextVariable(Smalltalk::context_inst_var_at(object, self.index))
                } else {
                    ReferencedObject::InstanceVariable(object.inst_var_at(self.index).unwrap(), self.index)
                };
                self.index += 1;

                return Some(next);
            }

            // an indexed object that can't have references to other objects is not interesting
            if object.header().format().is_bits() {
                return None;
            }

            // We ignore indexed fields weak objects
            if object.header().format().is_weak() {
                return None;
            }

            let total_amount = self.amount_of_fixed_fields + self.amount_of_indexable_fields;

            loop {
                if self.index >= total_amount {
                    break;
                }

                let index = self.index - self.amount_of_fixed_fields;
                let next = if self.is_context {
                    ReferencedObject::ContextVariable(Smalltalk::context_at(object, index + 1))
                } else {
                    let var = Smalltalk::item_at(
                        ObjectPointer::from(self.object.as_ptr()),
                        (index + 1).into(),
                    );
                    let var_ptr = RawObjectPointer::from(var.as_i64());
                    ReferencedObject::ArrayItem(AnyObjectRef::from(var_ptr))
                };

                self.index += 1;

                return Some(next);
            }
        }
        None
    }
}

fn amount_of_indexable_fields(oop: ObjectRef, length: usize) -> usize {
    length - amount_of_fixed_fields(oop, length)
}

fn amount_of_fixed_fields(oop: ObjectRef, length: usize) -> usize {
    match oop.header().format() {
        ObjectFormat::ZeroSized => length,
        ObjectFormat::NonIndexable => length,
        ObjectFormat::IndexableWithoutInstVars => 0,
        ObjectFormat::IndexableWithInstVars
        | ObjectFormat::WeakIndexable
        | ObjectFormat::WeakNonIndexable => {
            let class = Smalltalk::class_of_object(oop);
            let class_format_oop = Smalltalk::object_field_at(
                ObjectPointer::from(class.as_ptr()),
                ObjectFieldIndex::new(2),
            );
            let class_format =
                Immediate::try_from(RawObjectPointer::new(class_format_oop.as_i64()))
                    .unwrap()
                    .as_integer()
                    .unwrap();

            let mask = (1 << 16) - 1;
            class_format.bitand(mask) as usize
        }
        ObjectFormat::Forwarded => 0,
        ObjectFormat::Indexable64 => 0,
        ObjectFormat::Indexable32(_) => 0,
        ObjectFormat::Indexable16(_) => 0,
        ObjectFormat::Indexable8(_) => 0,
        ObjectFormat::CompiledMethod(_) => 0,
        ObjectFormat::Unsupported(_) => 0,
    }
}
