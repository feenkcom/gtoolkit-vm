mod error;
mod immediate;
mod object;
mod object_format;
mod object_header;
mod object_pointer;

pub use error::*;
pub use immediate::*;
pub use object::*;
pub use object_format::*;
pub use object_header::*;
pub use object_pointer::*;

#[macro_export]
macro_rules! assign_field {
    ($obj:ident . $field:ident, $value:expr) => {
        assign_field!($obj, $obj.$field, $value)
    };
    ($obj:expr, $setter:expr, $value:expr) => {{
        fn prepare_to_store_in(
            object: &vm_object_model::Object,
            value: impl Into<vm_object_model::AnyObjectRef>,
        ) {
            use vm_bindings::{ObjectPointer, Smalltalk};

            let object_ptr = ObjectPointer::from(object.as_ptr());
            let value_ptr = ObjectPointer::from(value.into().as_i64());
            Smalltalk::prepare_to_store(object_ptr, value_ptr);
        }

        let value_to_store = $value;
        prepare_to_store_in(&$obj, value_to_store);
        $setter = value_to_store;
    }};
}