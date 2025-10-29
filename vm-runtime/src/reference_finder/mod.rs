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
use vm_object_model::Immediate;

fn method_return_paths(
    paths: Vec<Vec<ReferencedObject>>,
    classes: ArrayRef,
) -> Result<(), anyhow::Error> {
    let paths = convert_referenced_object_paths(paths, classes)?;
    Smalltalk::method_return_value(ObjectPointer::from(paths.as_ptr()));
    Ok(())
}

fn method_return_path(path: Vec<ReferencedObject>, classes: ArrayRef) -> Result<(), anyhow::Error> {
    let paths = convert_referenced_object_path(path, classes)?;
    Smalltalk::method_return_value(ObjectPointer::from(paths.as_ptr()));
    Ok(())
}

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
        let mut inst = match each {
            ReferencedObject::InstanceVariable(_, index) => {
                let mut referenced_object =
                    Smalltalk::instantiate_class(instance_variable_class).as_object()?;
                referenced_object.inst_var_at_put(1, Immediate::new_u64(index as u64));
                referenced_object
            }
            ReferencedObject::ContextVariable(_) => {
                Smalltalk::instantiate_class(context_variable_class).as_object()?
            }
            ReferencedObject::ArrayItem(_) => {
                Smalltalk::instantiate_class(array_item_class).as_object()?
            }
            ReferencedObject::Root(_) => Smalltalk::instantiate_class(root_class).as_object()?,
        };

        Smalltalk::prepare_to_store(
            ObjectPointer::from(inst.as_ptr()),
            ObjectPointer::from(each.object().as_ptr()),
        );
        inst.inst_var_at_put(0, each.object());

        array.insert(index, inst);
    }

    Ok(array)
}
