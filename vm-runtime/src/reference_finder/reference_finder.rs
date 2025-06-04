use vm_object_model::{AnyObjectRef, Immediate, ObjectFormat, ObjectRef, RawObjectPointer};

use crate::objects::Array;
use crate::reference_finder::find_first_path_with_backlinks::find_paths_with_backlinks;
use num_traits::Zero;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::BitAnd;
use vm_bindings::{ObjectFieldIndex, ObjectPointer, Smalltalk, StackOffset};

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveReferenceFinderFindAllPaths() {
    let start_obj = Smalltalk::stack_ref(StackOffset::new(0));
    let target_obj = Smalltalk::stack_ref(StackOffset::new(1));

    let paths = find_paths_with_backlinks(start_obj, target_obj, true);

    let mut paths_array = Array::new(paths.len()).unwrap();
    for (path_index, path) in paths.iter().enumerate() {
        let mut path_array = Array::new(path.len()).unwrap();
        for (obj_index, obj) in path.iter().enumerate() {
            path_array.insert(obj_index, obj.clone());
        }
        paths_array.insert(path_index, path_array.clone());
    }

    Smalltalk::method_return_value(ObjectPointer::from(paths_array.as_ptr()));
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveReferenceFinderFindPath() {
    let start_obj = Smalltalk::stack_ref(StackOffset::new(0));
    let target_obj = Smalltalk::stack_ref(StackOffset::new(1));

    let path = find_paths_with_backlinks(start_obj, target_obj, false)
        .pop()
        .unwrap_or(vec![]);

    let mut array = Array::new(path.len()).unwrap();
    for (index, each) in path.iter().enumerate() {
        array.insert(index, each.clone());
    }

    Smalltalk::method_return_value(ObjectPointer::from(array.as_ptr()));
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveReferenceFinderGetNeighbours() {
    let start_obj = Smalltalk::stack_ref(StackOffset::new(0));

    let mut neighbors = start_obj.neighbors(start_obj);
    neighbors.include_all = true;
    let neighbors: Vec<AnyObjectRef> = neighbors.collect();
    let mut array = Array::new(neighbors.len()).unwrap();
    for (index, each) in neighbors.iter().enumerate() {
        array.insert(index, each.clone());
    }

    Smalltalk::method_return_value(ObjectPointer::from(array.as_ptr()));
}

impl GraphNode for AnyObjectRef {
    fn neighbors(&self, target: AnyObjectRef) -> ObjectIterator {
        ObjectIterator::new(self.clone(), target)
    }
}

pub trait GraphNode: Debug + Clone + Eq + Hash {
    fn neighbors(&self, target: AnyObjectRef) -> ObjectIterator;
}

#[derive(Debug)]
pub struct ObjectIterator {
    pub object: AnyObjectRef,
    pub target: AnyObjectRef,
    pub amount_of_fixed_fields: usize,
    pub amount_of_indexable_fields: usize,
    pub index: usize,
    pub is_context: bool,
    pub include_all: bool
}

impl ObjectIterator {
    pub fn new(oop: AnyObjectRef, target: AnyObjectRef) -> Self {
        if let Ok(object) = oop.as_object() {
            let amount_of_units = object.amount_of_indexable_units();
            let amount_of_fixed_fields = amount_of_fixed_fields(object, amount_of_units);

            if object.is_context() {
                return ObjectIterator {
                    object: oop,
                    target,
                    amount_of_fixed_fields,
                    amount_of_indexable_fields: Smalltalk::context_size(object),
                    index: 0,
                    is_context: true,
                    include_all: false,
                };
            }

            let amount_of_indexable_fields = amount_of_indexable_fields(object, amount_of_units);
            let is_ephemeron = object.header().format().is_ephemeron();
            let start_index = if is_ephemeron { 1 } else { 0 };

            ObjectIterator {
                object: oop,
                target,
                amount_of_fixed_fields,
                amount_of_indexable_fields,
                index: start_index,
                is_context: false,
                include_all: false,
            }
        } else {
            ObjectIterator {
                object: oop,
                target,
                amount_of_fixed_fields: 0,
                amount_of_indexable_fields: 0,
                index: 0,
                is_context: false,
                include_all: false,
            }
        }
    }

    fn is_interesting_object(&self, another_object: AnyObjectRef) -> bool {
        if self.include_all {
            return true;
        }

        if self.target == another_object {
            return true;
        }

        if let Ok(another_object) = another_object.as_object() {
            // empty objects are not interesting
            if another_object.amount_of_indexable_units().is_zero() {
                return false;
            }
            true
        } else {
            false
        }
    }
}

impl Iterator for ObjectIterator {
    type Item = AnyObjectRef;
    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(object) = self.object.as_object() {
            loop {
                if self.index >= self.amount_of_fixed_fields {
                    break;
                }

                let next = if self.is_context {
                    Smalltalk::context_inst_var_at(object, self.index)
                } else {
                    object.inst_var_at(self.index).unwrap()
                };
                self.index += 1;

                if self.is_interesting_object(next) {
                    return Some(next);
                }
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
                    Smalltalk::context_at(object, index + 1)
                } else {
                    let var = Smalltalk::item_at(
                        ObjectPointer::from(self.object.as_ptr()),
                        (index + 1).into(),
                    );
                    let var_ptr = RawObjectPointer::from(var.as_i64());
                    AnyObjectRef::from(var_ptr)
                };

                self.index += 1;

                if self.is_interesting_object(next) {
                    return Some(next);
                }
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
