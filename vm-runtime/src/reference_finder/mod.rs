mod instance_counter;
mod object_iterator;
mod object_visitor;
mod reference_finder;

use crate::objects::{Array, ArrayRef};
use anyhow::anyhow;
pub use instance_counter::*;
pub use object_iterator::*;
pub use object_visitor::*;
pub use reference_finder::*;
use vm_bindings::{ObjectPointer, Smalltalk};
use vm_object_model::{AnyObjectRef, ObjectRef};

fn convert_referenced_object_paths(
    paths: Vec<Vec<ReferencedObject>>,
    classes: ArrayRef,
) -> Result<ArrayRef, anyhow::Error> {
    let mut paths_array = Array::new(paths.len())?;
    for (path_index, path) in paths.into_iter().enumerate() {
        let path_array = convert_referenced_object_path(path, classes)?;
        paths_array.insert(path_index, path_array);
    }

    Ok(paths_array)
}

fn convert_referenced_object_path(
    path: Vec<ReferencedObject>,
    classes: ArrayRef,
) -> Result<ArrayRef, anyhow::Error> {
    let root_class = classes
        .get(0)
        .ok_or_else(|| anyhow!("Root class is not defined"))?
        .as_object()?;
    let instance_variable_class = classes
        .get(1)
        .ok_or_else(|| anyhow!("Instance variable class is not defined"))?
        .as_object()?;
    let context_variable_class = classes
        .get(2)
        .ok_or_else(|| anyhow!("Context variable class is not defined"))?
        .as_object()?;

    let array_item_class = classes
        .get(3)
        .ok_or_else(|| anyhow!("Array item class is not defined"))?
        .as_object()?;

    let mut array = Array::new(path.len())?;
    for (index, each) in path.into_iter().enumerate() {
        let inst_class = match each {
            ReferencedObject::InstanceVariable(_) => instance_variable_class,
            ReferencedObject::ContextVariable(_) => context_variable_class,
            ReferencedObject::ArrayItem(_) => array_item_class,
            ReferencedObject::Root(_) => root_class,
        };

        let mut inst = Smalltalk::instantiate_class(inst_class).as_object()?;
        inst.inst_var_at_put(0, each.object());

        array.insert(index, inst);
    }

    Ok(array)
}

fn method_return_path(path: Vec<ReferencedObject>) {
    let mut array = Array::new(path.len()).unwrap();
    for (index, each) in path.iter().enumerate() {
        array.insert(index, each.object());
    }

    Smalltalk::method_return_value(ObjectPointer::from(array.as_ptr()));
}

fn method_return_paths(paths: Vec<Vec<AnyObjectRef>>) {
    let mut paths_array = Array::new(paths.len()).unwrap();
    for (path_index, path) in paths.iter().enumerate() {
        let mut path_array = Array::new(path.len()).unwrap();
        for (obj_index, obj) in path.iter().enumerate() {
            path_array.insert(obj_index, *obj);
        }
        paths_array.insert(path_index, path_array);
    }

    Smalltalk::method_return_value(ObjectPointer::from(paths_array.as_ptr()));
}
