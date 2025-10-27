use crate::objects::{ByteSymbol, CompiledMethod, WeakSymbolSet, WeakSymbolSetRef};
use crate::vm;
use pharo_compiler::bytecode::CompiledCodeLiteral;
use pharo_compiler::ir::{OwnedLiteral, OwnedLiteralValue};
use pharo_compiler::kernel_environment;
use pharo_compiler::vm_plugin::PharoCompiler;
use std::fmt::Debug;
use std::ops::Deref;
use std::slice;
use vm_bindings::{Smalltalk, StackOffset};
use vm_object_model::{AnyObjectRef, ObjectRef, RawObjectPointer};

#[cfg(not(feature = "pharo-compiler"))]
compile_error!("\"pharo-compiler\" feature must be enabled for this module.");

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitivePharoCompilerNew() {
    let proxy = vm().proxy();
    let compiled_method_class = Smalltalk::stack_object_value_unchecked(StackOffset::new(1));
    let symbol_table = Smalltalk::stack_object_value_unchecked(StackOffset::new(0));

    let compiler = PharoCompiler {
        environment: kernel_environment(),
        compiled_method_class: compiled_method_class.into(),
        symbol_table: symbol_table.into(),
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

    let compiler_external_object = Smalltalk::stack_object_value(StackOffset::new(0)).unwrap();
    let compiler_ptr = proxy.read_address(compiler_external_object);
    let compiler: &PharoCompiler = unsafe { &*(compiler_ptr as *const PharoCompiler) };

    let source = Smalltalk::stack_object_value(StackOffset::new(1)).unwrap();
    let source_size = Smalltalk::size_of(source);
    let source_ptr = Smalltalk::first_indexable_field(source) as *const u8;

    let source_bytes = unsafe { slice::from_raw_parts(source_ptr, source_size) };
    let source_str = unsafe { std::str::from_utf8_unchecked(source_bytes) };

    let compiled_method = compiler.compile(source_str);

    let compiled_method_object =
        AnyObjectRef::from(RawObjectPointer::from(compiled_method.pharo_method));
    let compiled_method_object = compiled_method_object.as_object().unwrap();
    let compiled_method_object = CompiledMethod::try_from(compiled_method_object.deref()).unwrap();

    let symbol_table = AnyObjectRef::from(RawObjectPointer::from(compiler.symbol_table));
    let symbol_table = WeakSymbolSetRef::try_from(symbol_table).unwrap();

    println!("Compiled!");

    for (index, literal) in compiled_method.literals().iter().enumerate() {
        match literal {
            CompiledCodeLiteral::Literal(literal) => match literal {
                OwnedLiteral::Value(literal) => match literal {
                    OwnedLiteralValue::True => {}
                    OwnedLiteralValue::False => {}
                    OwnedLiteralValue::Nil => {}
                    OwnedLiteralValue::Integer(_) => {}
                    OwnedLiteralValue::Float(_) => {}
                    OwnedLiteralValue::Character(_) => {}
                    OwnedLiteralValue::String(string) => {
                        let smalltalk_string = proxy.new_string(string.as_str());
                        println!("smalltalk_string: {:?}", smalltalk_string);
                        compiled_method_object.set_literal(
                            AnyObjectRef::from(RawObjectPointer::from(smalltalk_string.as_i64())),
                            index,
                        );
                    }
                    OwnedLiteralValue::Symbol(string) => {
                        if let Some(symbol) = symbol_table.find_like_byte_str(string) {
                            compiled_method_object.set_literal(AnyObjectRef::from(symbol), index);
                        }
                    }
                    OwnedLiteralValue::ConstantBlockClosure => {}
                },
                OwnedLiteral::Array(_) => {}
                OwnedLiteral::ByteArray(_) => {}
            },
            CompiledCodeLiteral::Variable(_) => {}
            CompiledCodeLiteral::CompiledBlock(_) => {}
        }
    }

    Smalltalk::method_return_value(compiled_method.pharo_method.into());
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitivePharoCompilerPrintObject() {
    // let smalltalk = Smalltalk::new();
    //
    // let object = smalltalk.get_stack_value(StackOffset::new(0));
    // println!("object: {:#?}", &object);
    //
    // if let Some(object) = object.as_object().ok() {
    //     if let Some(array) = object.try_as_array() {
    //         println!("{:?}", array.items().collect::<Vec<_>>());
    //     }
    // }

    // let immediate_masked = Into::<sqInt>::into(object) & 7;
    // println!("object {:?}", object);
    // println!("immediate_masked {}", immediate_masked);
    //
    // if object.is_immediate() {
    //     println!("immediate {:?}", object);
    // }
    // else {
    //     let raw_header: *mut ObjectHeader = unsafe { std::mem::transmute(object) };
    //     let object_header = unsafe { &*raw_header };
    //     println!("{:#?}", object_header);
    // }

    Smalltalk::method_return_boolean(true);
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitivePharoCompilerFindInWeakSet() {
    let smalltalk = Smalltalk::new();

    let weak_set = smalltalk.get_stack_value(StackOffset::new(1));

    let string = smalltalk
        .get_stack_value(StackOffset::new(0))
        .as_object()
        .unwrap();

    let weak_set = WeakSymbolSetRef::try_from(weak_set).unwrap();

    let byte_string = ByteSymbol::try_from(string.deref()).unwrap();

    if let Some(item) = weak_set.find_like_byte_str(byte_string.as_str()) {
        Smalltalk::method_return_value(item.as_ptr().into());
    } else {
        Smalltalk::method_return_value(Smalltalk::primitive_nil_object());
    }
}
