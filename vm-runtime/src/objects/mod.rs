mod array;
mod association;
mod byte_symbol;
mod compiled_method;
pub mod identity_dictionary;
mod ordered_collection;
mod weak_symbol_set;
mod wide_symbol;

pub use array::*;
pub use association::*;
pub use byte_symbol::*;
pub use compiled_method::*;
pub use identity_dictionary::*;
pub use ordered_collection::*;
pub use weak_symbol_set::*;
pub use wide_symbol::*;

#[macro_export]
macro_rules! assign_field {
    ($obj:ident . $field:ident, $value:expr) => {
        assign_field!($obj, $obj . $field, $value)
    };
    ($obj:expr, $setter:expr, $value:expr) => {
        {
            fn prepare_to_store_in(object: &vm_object_model::Object, value: impl Into<vm_object_model::AnyObjectRef>) {
                use vm_bindings::{ObjectPointer, Smalltalk};

                let object_ptr = ObjectPointer::from(object.as_ptr());
                let value_ptr = ObjectPointer::from(value.into().as_i64());
                Smalltalk::prepare_to_store(object_ptr, value_ptr);
            }

            let value_to_store = $value;
            prepare_to_store_in(&$obj, value_to_store);
            $setter = value_to_store;
        }
    };
}

