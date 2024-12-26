use crate::vm;
use pharo_compiler::kernel_environment;
use pharo_compiler::vm_plugin::PharoCompiler;
use std::slice;
use vm_bindings::{Smalltalk, StackOffset};

#[cfg(not(feature = "pharo-compiler"))]
compile_error!("\"pharo-compiler\" feature must be enabled for this module.");

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitivePharoCompilerNew() {
    let proxy = vm().proxy();
    let compiled_method_class = Smalltalk::stack_object_value(StackOffset::new(0));

    let compiler = PharoCompiler {
        environment: kernel_environment(),
        compiled_method_class: compiled_method_class.into(),
        vm_create_new_compiled_method: vm_bindings::bindings::createNewMethodheaderbytecodeCount,
        vm_method_return_value: vm_bindings::bindings::methodReturnValue,
        vm_object_size: vm_bindings::bindings::stSizeOf,
        vm_object_at_put: vm_bindings::bindings::stObjectatput,
        vm_object_first_indexable_field: vm_bindings::bindings::firstIndexableField,
        vm_object_first_fixed_field: vm_bindings::bindings::firstFixedField,
        vm_integer_object_of: vm_bindings::bindings::integerObjectOf,
    };

    let boxed_compiler = Box::into_raw(Box::new(compiler));
    let compiler_address = proxy.new_external_address(boxed_compiler);
    Smalltalk::method_return_value(compiler_address);
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitivePharoCompilerCompile() {
    let proxy = vm().proxy();

    let compiler_external_object = Smalltalk::stack_object_value(StackOffset::new(0));
    let compiler_ptr = proxy.read_address(compiler_external_object);
    let compiler: &PharoCompiler = unsafe { &*(compiler_ptr as *const PharoCompiler) };

    let source = Smalltalk::stack_object_value(StackOffset::new(1));
    let source_size = Smalltalk::size_of(source);
    let source_ptr = Smalltalk::first_indexable_field(source) as *const u8;

    let source_bytes = unsafe { slice::from_raw_parts(source_ptr, source_size) };
    let source_str = unsafe { std::str::from_utf8_unchecked(source_bytes) };

    let compiled_method = compiler.compile(source_str);

    Smalltalk::method_return_value(compiled_method.pharo_method.into());
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitivePharoCompilerPrintObject() {
    let object = Smalltalk::stack_object_value(StackOffset::new(0));
    object.read_uint32();
}
