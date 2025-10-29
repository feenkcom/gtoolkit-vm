use std::cell::RefCell;
use std::ffi::{c_int, CString};
use std::mem::transmute;
use std::ops::Deref;
use std::os::raw::c_void;
use std::process::exit;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread::JoinHandle;

use crate::objects::{Array, ArrayRef};
#[cfg(feature = "pharo-compiler")]
use crate::pharo_compiler::*;
#[cfg(feature = "tonel")]
use crate::tonel::*;

use crate::reference_finder::{
    primitiveClassInstanceReferenceFinderFindAllPaths,
    primitiveClassInstanceReferenceFinderFindPath, primitiveInstanceCounterCountAll,
    primitiveReferenceFinderFindAllPaths, primitiveReferenceFinderFindPath,
    primitiveReferenceFinderGetNeighbours,
};
use crate::version::{app_info, app_version};
#[cfg(feature = "ffi")]
use crate::{
    ffi::{
        primitiveBareFfiCallout, primitiveBareFfiCalloutInvalidate, primitiveBareFfiCalloutRelease,
    },
    primitiveEventLoopCallout, primitiveExtractReturnValue,
};
use crate::{
    log_signal, primitiveEnableLogSignal, primitiveGetEnabledLogSignals, primitivePollLogger,
    primitiveStartBeacon, primitiveStartConsoleLogger, primitiveStartGlobalProcessSwitchTelemetry,
    primitiveStartLocalProcessSwitchTelemetry, primitiveStopLogger, primitiveStopTelemetry,
    primitiveTelemetryContextSignal, primitiveTelemetryObjectSignal, should_log_all_signals,
    should_log_signal, ConsoleLogger, EventLoop, EventLoopMessage, EventLoopWaker, VM_LOGGER,
};
use anyhow::Result;
use vm_bindings::{
    virtual_machine_info, InterpreterConfiguration, InterpreterProxy, LogLevel, NamedPrimitive,
    ObjectFieldIndex, ObjectPointer, PharoInterpreter, Smalltalk, StackOffset,
};
use vm_object_model::{AnyObjectRef, Error, RawObjectPointer};
use widestring::U32Str;

#[no_mangle]
pub static mut VIRTUAL_MACHINE: Option<Arc<VirtualMachine>> = None;
pub fn vm() -> &'static Arc<VirtualMachine> {
    unsafe { VIRTUAL_MACHINE.as_ref().expect("VM must be initialized") }
}

#[derive(Debug)]
pub struct VirtualMachine {
    interpreter: Arc<PharoInterpreter>,
    event_loop: Option<EventLoop>,
    event_loop_sender: Option<Sender<EventLoopMessage>>,
    event_loop_waker: RefCell<Option<EventLoopWaker>>,
    #[cfg(target_os = "android")]
    android_app: android_activity::AndroidApp,
}

#[derive(Debug)]
pub struct VirtualMachineConfiguration {
    pub interpreter_configuration: InterpreterConfiguration,
    /// When None - log nothing.
    /// When Some with an empty list - log everything.
    /// When Some with a list of signal name - log only those
    pub log_signals: Option<Vec<String>>,
}

impl VirtualMachine {
    /// Create a Virtual Machine for given interpreter parameters.
    /// If event loop sender is `None` - start the virtual machine
    /// in the main thread rather than the worker thread
    pub fn new(
        configuration: VirtualMachineConfiguration,
        event_loop: Option<EventLoop>,
        event_loop_sender: Option<Sender<EventLoopMessage>>,
        #[cfg(target_os = "android")] android_app: android_activity::AndroidApp,
    ) -> Self {
        let vm = Self {
            interpreter: Arc::new(PharoInterpreter::new(
                configuration.interpreter_configuration,
            )),
            event_loop,
            event_loop_sender,
            event_loop_waker: RefCell::new(None),
            #[cfg(target_os = "android")]
            android_app,
        };

        if let Some(signals) = configuration.log_signals {
            let mut logger = VM_LOGGER.lock().unwrap();
            logger.set_logger(Box::new(ConsoleLogger::new()));

            // this one configures Pharo VM to log via our `VM_LOGGER`
            vm.interpreter().set_logger(Some(log_signal));

            if signals.is_empty() {
                vm.interpreter()
                    .set_should_log(Some(should_log_all_signals));
            } else {
                vm.interpreter().set_should_log(Some(should_log_signal));
                for signal in signals {
                    logger.enable_type(CString::new(signal).unwrap());
                }
            }
        }

        #[cfg(feature = "ffi")]
        {
            vm.add_primitive(primitive!(primitiveGetNamedPrimitives));
            vm.add_primitive(primitive!(primitiveEventLoopCallout));
            vm.add_primitive(primitive!(primitiveExtractReturnValue));
            
            vm.add_primitive(try_primitive!(primitiveBareFfiCallout));
            vm.add_primitive(try_primitive!(primitiveBareFfiCalloutInvalidate));
            vm.add_primitive(try_primitive!(primitiveBareFfiCalloutRelease));
        }

        #[cfg(feature = "tonel")]
        {
            use crate::tonel::*;
            vm.add_primitive(try_primitive!(primitiveTonelBuildLoadPlan));
        }
        
        vm.add_primitive(primitive!(primitiveGetSemaphoreSignaller));
        vm.add_primitive(primitive!(primitiveGetEventLoop));
        vm.add_primitive(primitive!(primitiveGetEventLoopReceiver));
        vm.add_primitive(primitive!(primitiveStopLogger));
        vm.add_primitive(primitive!(primitivePollLogger));
        vm.add_primitive(primitive!(primitiveEnableLogSignal));
        vm.add_primitive(primitive!(primitiveGetEnabledLogSignals));
        vm.add_primitive(primitive!(primitiveStartBeacon));
        vm.add_primitive(primitive!(primitiveStartConsoleLogger));
        vm.add_primitive(primitive!(primitiveSetEventLoopWaker));
        vm.add_primitive(primitive!(primitiveFullGarbageCollectorMicroseconds));
        vm.add_primitive(primitive!(primitiveScavengeGarbageCollectorMicroseconds));
        vm.add_primitive(primitive!(primitiveFirstBytePointerOfDataObject));
        vm.add_primitive(primitive!(primitivePointerAtPointer));
        vm.add_primitive(primitive!(primitiveFcntl));
        vm.add_primitive(primitive!(primitiveVirtualMachineInfo));
        vm.add_primitive(primitive!(primitiveAppInfo));
        vm.add_primitive(primitive!(primitiveAppVersion));
        vm.add_primitive(primitive!(primitiveWideStringByteIndexToCharIndex));
        vm.add_primitive(primitive!(primitiveIdentityDictionaryScanFor));
        vm.add_primitive(primitive!(primitiveIdentityHash));

        // telemetry
        vm.add_primitive(primitive!(primitiveStartLocalProcessSwitchTelemetry));
        vm.add_primitive(primitive!(primitiveStartGlobalProcessSwitchTelemetry));
        vm.add_primitive(primitive!(primitiveTelemetryObjectSignal));
        vm.add_primitive(primitive!(primitiveTelemetryContextSignal));
        vm.add_primitive(primitive!(primitiveStopTelemetry));

        // reference finder
        vm.add_primitive(try_primitive!(primitiveReferenceFinderFindAllPaths));
        vm.add_primitive(try_primitive!(primitiveReferenceFinderFindPath));
        vm.add_primitive(try_primitive!(primitiveReferenceFinderGetNeighbours));
        vm.add_primitive(try_primitive!(
            primitiveClassInstanceReferenceFinderFindAllPaths
        ));
        vm.add_primitive(try_primitive!(
            primitiveClassInstanceReferenceFinderFindPath
        ));
        vm.add_primitive(primitive!(primitiveInstanceCounterCountAll));

        vm.add_primitive(primitive!(primitiveIsOldObject));
        vm.add_primitive(primitive!(primitiveIsYoungObject));

        // debug
        vm.add_primitive(primitive!(primitiveDebugPrintArray));

        #[cfg(feature = "pharo-compiler")]
        {
            vm.add_primitive(primitive!(primitivePharoCompilerNew));
            vm.add_primitive(primitive!(primitivePharoCompilerCompile));
            vm.add_primitive(primitive!(primitivePharoCompilerPrintObject));
            vm.add_primitive(primitive!(primitivePharoCompilerFindInWeakSet));
        }

        #[cfg(target_os = "android")]
        vm.add_primitive(primitive!(primitiveGetAndroidApp));
        #[cfg(target_os = "android")]
        vm.add_primitive(primitive!(primitiveGetAndroidNativeWindow));
        vm
    }

    /// Register a given named primitive in the interpreter
    pub fn add_primitive(&self, primitive: NamedPrimitive) {
        self.interpreter.add_vm_export(primitive);
    }

    /// Return a slice of all registered named primitives in the vm
    pub fn named_primitives(&self) -> &[NamedPrimitive] {
        self.interpreter.vm_exports()
    }

    /// Return a proxy to the interpreter which can be used to communicate with the vm
    /// *Expensive*, as it allocates a large struct on each call
    pub fn proxy(&self) -> &InterpreterProxy {
        self.interpreter.proxy()
    }

    pub fn interpreter(&self) -> &PharoInterpreter {
        &self.interpreter
    }

    /// Launch the virtual machine either in the main thread or in the worker thread
    /// depending on how the virtual machine was instantiated.
    pub fn start(&self) -> Result<Option<JoinHandle<Result<()>>>> {
        self.interpreter.clone().start()
    }

    pub fn event_loop(&self) -> Option<&EventLoop> {
        self.event_loop.as_ref()
    }

    /// Register this virtual machine in a global variable. There can only be one virtual machine running in one memory space
    pub fn register(self: Arc<Self>) {
        unsafe { VIRTUAL_MACHINE = Some(self) };
    }

    pub fn send(&self, message: EventLoopMessage) -> Result<()> {
        if let Some(sender) = self.event_loop_sender.as_ref() {
            sender.send(message).unwrap();

            if let Some(waker) = self.event_loop_waker.borrow().as_ref() {
                waker.wake();
            }
        } else {
            match message {
                #[cfg(feature = "ffi")]
                EventLoopMessage::Call(callout) => {
                    callout.lock().unwrap().call();
                }
                EventLoopMessage::Terminate => {
                    exit(0);
                }
                EventLoopMessage::WakeUp => {}
            }
        }
        Ok(())
    }
}

#[no_mangle]
pub fn is_virtual_machine() {
    Smalltalk::method_return_boolean(true);
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveGetNamedPrimitives() {
    let proxy = vm().proxy();
    let named_primitives = vm().named_primitives();

    let return_array = Smalltalk::primitive_instantiate_indexable_class_of_size(
        Smalltalk::primitive_class_array(),
        named_primitives.len(),
    );
    for (index, named_primitive) in named_primitives.iter().enumerate() {
        let each_primitive_array = Smalltalk::primitive_instantiate_indexable_class_of_size(
            Smalltalk::primitive_class_array(),
            3,
        );

        let plugin_name = proxy.new_string(named_primitive.plugin_name());
        let primitive_name = proxy.new_string(named_primitive.primitive_name());
        let primitive_address =
            Smalltalk::new_external_address(named_primitive.primitive_address());

        Smalltalk::item_at_put(each_primitive_array, ObjectFieldIndex::new(1), plugin_name);
        Smalltalk::item_at_put(
            each_primitive_array,
            ObjectFieldIndex::new(2),
            primitive_name,
        );
        Smalltalk::item_at_put(
            each_primitive_array,
            ObjectFieldIndex::new(3),
            primitive_address,
        );

        Smalltalk::item_at_put(
            return_array,
            ObjectFieldIndex::new(index + 1),
            each_primitive_array,
        );
    }
    Smalltalk::method_return_value(return_array);
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveGetSemaphoreSignaller() {
    let signaller = Smalltalk::new_external_address(semaphore_signaller as *const c_void);
    Smalltalk::method_return_value(signaller);
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveGetEventLoop() {
    let vm = vm();

    let event_loop = match vm.event_loop() {
        None => std::ptr::null(),
        Some(event_loop) => event_loop as *const EventLoop,
    };

    Smalltalk::method_return_value(Smalltalk::new_external_address(event_loop));
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveGetEventLoopReceiver() {
    let receiver = Smalltalk::new_external_address(try_receive_events as *const c_void);
    Smalltalk::method_return_value(receiver);
}

#[no_mangle]
#[allow(non_snake_case)]
#[cfg(target_os = "android")]
/// Returns a reference to a clone of the AndroidApp.
/// Users are responsible for managing the memory
pub fn primitiveGetAndroidApp() {
    let vm = vm();

    let android_app = Box::into_raw(Box::new(vm.android_app.clone()));
    Smalltalk::method_return_value(Smalltalk::new_external_address(android_app));
}

#[no_mangle]
#[allow(non_snake_case)]
#[cfg(target_os = "android")]
/// Returns a pointer to the NativeWindow
pub fn primitiveGetAndroidNativeWindow() {
    let vm = vm();

    let native_window = vm
        .android_app
        .native_window()
        .map(|native_window| native_window.ptr().as_ptr())
        .unwrap_or(std::ptr::null_mut());

    Smalltalk::method_return_value(Smalltalk::new_external_address(native_window));
}

#[no_mangle]
pub extern "C" fn try_receive_events(event_loop_ptr: *const EventLoop) {
    if event_loop_ptr.is_null() {
        return;
    }
    let event_loop = unsafe { &*event_loop_ptr };
    event_loop.try_recv().unwrap();
}

#[no_mangle]
pub extern "C" fn semaphore_signaller(semaphore_index: usize) {
    vm().proxy().signal_semaphore(semaphore_index);
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveSetEventLoopWaker() {
    let waker_thunk_external_address = Smalltalk::stack_ref(StackOffset::new(1))
        .as_object()
        .unwrap();
    let waker_function_external_address = Smalltalk::stack_ref(StackOffset::new(0))
        .as_object()
        .unwrap();

    let waker_function_ptr = Smalltalk::read_external_address(waker_function_external_address);

    let waker_function: extern "C" fn(*const c_void, u32) -> bool =
        unsafe { transmute(waker_function_ptr) };
    let waker_thunk = Smalltalk::read_external_address(waker_thunk_external_address);

    let waker = EventLoopWaker::new(waker_function, waker_thunk);
    vm().event_loop_waker.replace(Some(waker));

    Smalltalk::method_return_boolean(true);
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveFullGarbageCollectorMicroseconds() {
    let proxy = vm().proxy();

    let microseconds = vm().interpreter().full_gc_microseconds();
    Smalltalk::method_return_value(proxy.new_positive_64bit_integer(microseconds));
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveScavengeGarbageCollectorMicroseconds() {
    let proxy = vm().proxy();

    let microseconds = vm().interpreter().scavenge_gc_microseconds();
    Smalltalk::method_return_value(proxy.new_positive_64bit_integer(microseconds));
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveFirstBytePointerOfDataObject() {
    let receiver = Smalltalk::stack_object_value_unchecked(StackOffset::new(0));

    let pointer = Smalltalk::first_byte_pointer_of_data_object(receiver);
    Smalltalk::method_return_value(Smalltalk::new_external_address(pointer));
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitivePointerAtPointer() {
    let external_address = Smalltalk::stack_ref(StackOffset::new(0))
        .as_object()
        .unwrap();
    let external_address_pointer = Smalltalk::read_external_address(external_address);
    let pointer = Smalltalk::pointer_at_pointer(external_address_pointer);
    Smalltalk::method_return_value(Smalltalk::new_external_address(pointer));
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveVirtualMachineInfo() {
    let proxy = vm().proxy();
    let info = virtual_machine_info();
    Smalltalk::method_return_value(proxy.new_string(info));
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveAppInfo() {
    let proxy = vm().proxy();
    let info = app_info();
    Smalltalk::method_return_value(proxy.new_string(info));
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveAppVersion() {
    let proxy = vm().proxy();
    let version = app_version();
    Smalltalk::method_return_value(proxy.new_string(version));
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveFcntl() {
    #[cfg(unix)]
    {
        match Smalltalk::method_argument_count() {
            2 => {
                let file_descriptor = Smalltalk::stack_integer_value(StackOffset::new(1)) as c_int;
                let command = Smalltalk::stack_integer_value(StackOffset::new(0)) as c_int;

                let result = unsafe { libc::fcntl(file_descriptor, command) };
                Smalltalk::method_return_value(Smalltalk::new_integer_pointer(result));
            }
            3 => {
                let file_descriptor = Smalltalk::stack_integer_value(StackOffset::new(2)) as c_int;
                let command = Smalltalk::stack_integer_value(StackOffset::new(1)) as c_int;
                let argument = Smalltalk::stack_integer_value(StackOffset::new(0)) as c_int;

                let result = unsafe { libc::fcntl(file_descriptor, command, argument) };
                Smalltalk::method_return_value(Smalltalk::new_integer_pointer(result));
            }
            _ => Smalltalk::primitive_fail(),
        }
    }
    #[cfg(not(unix))]
    {
        Smalltalk::primitive_fail();
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveWideStringByteIndexToCharIndex() {
    if Smalltalk::method_argument_count() != 1 {
        error!(
            "Wrong argument count, expected 1 got {}",
            Smalltalk::method_argument_count()
        );
        Smalltalk::primitive_fail();
        return;
    }

    let byte_offset = Smalltalk::stack_integer_value(StackOffset::new(0)) as usize;

    let wide_string = Smalltalk::stack_object_value_unchecked(StackOffset::new(1));
    let wide_string_size = Smalltalk::size_of(wide_string);

    let wide_string_ptr = Smalltalk::first_indexable_field(wide_string) as *const u32;

    let u32_str = unsafe { U32Str::from_ptr(wide_string_ptr, wide_string_size) };
    let mut char_offset = 1usize;
    let mut current_byte_offset = 0usize;
    for each in u32_str.chars_lossy() {
        current_byte_offset += each.len_utf8();
        if current_byte_offset > byte_offset {
            Smalltalk::method_return_integer(char_offset as i64);
            return;
        }
        char_offset += 1;
    }
    Smalltalk::method_return_integer(char_offset.min(wide_string_size) as i64);
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveIdentityHash() {
    let object = Smalltalk::stack_object_value(StackOffset::new(0)).unwrap();
    let hash = Smalltalk::identity_hash(object);
    Smalltalk::method_return_integer(hash as i64)
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveIdentityDictionaryScanFor() {
    if Smalltalk::method_argument_count() != 2 {
        error!(
            "Wrong argument count, expected 2 got {}",
            Smalltalk::method_argument_count()
        );
        Smalltalk::primitive_fail();
        return;
    }

    let hash = Smalltalk::stack_integer_value(StackOffset::new(0)) as u32;
    let object = Smalltalk::stack_object_value_unchecked(StackOffset::new(1));
    let dictionary = Smalltalk::stack_object_value_unchecked(StackOffset::new(2));

    let array = Smalltalk::object_field_at(dictionary, 1usize.into());

    let finish = Smalltalk::size_of(array) as u32;
    let start = hash.rem_euclid(finish) + 1;

    fn find_item_or_empty_slot(
        start: u32,
        finish: u32,
        array: ObjectPointer,
        object: ObjectPointer,
    ) -> Option<u32> {
        let nil_object = Smalltalk::primitive_nil_object();

        // Search from (hash mod size) to the end.
        for index in start..(finish + 1) {
            let association = Smalltalk::item_at(array, index.into());
            if Smalltalk::is_identical(association, nil_object)? {
                return Some(index);
            }

            let key = Smalltalk::object_field_at(association, 0usize.into());
            if Smalltalk::is_identical(key, object)? {
                return Some(index);
            }
        }

        // Search from 1 to where we started.
        for index in 1..start {
            let association = Smalltalk::item_at(array, index.into());
            if Smalltalk::is_identical(association, nil_object)? {
                return Some(index);
            }

            let key = Smalltalk::object_field_at(association, 0usize.into());
            if Smalltalk::is_identical(key, object)? {
                return Some(index);
            }
        }

        Some(0)
    }

    match find_item_or_empty_slot(start, finish, array, object) {
        None => Smalltalk::primitive_fail_code(1),
        Some(index) => Smalltalk::method_return_integer(index as i64),
    }
}

// #[no_mangle]
// #[allow(non_snake_case)]
// pub fn primitiveIdentityDictionaryScanFor2() {
//     if Smalltalk::method_argument_count() != 2 {
//         error!(
//             "Wrong argument count, expected 2 got {}",
//             Smalltalk::method_argument_count()
//         );
//         Smalltalk::primitive_fail();
//         return;
//     }
//
//     let hash = Smalltalk::stack_integer_value(StackOffset::new(0)) as u32;
//     let object_ptr = Smalltalk::stack_object_value_unchecked(StackOffset::new(1));
//     let object_raw = RawObjectPointer::new(object_ptr.into());
//     let object = object_raw.reify();
//     let dictionary = Smalltalk::stack_object_value_unchecked(StackOffset::new(2));
//
//     let array_ptr = Smalltalk::object_field_at(dictionary, 1usize.into());
//     let raw_pointer = RawObjectPointer::new(array_ptr.into());
//     let array = raw_pointer
//         .reify()
//         .as_object_unchecked()
//         .try_as_array()
//         .unwrap();
//
//     let nil_ptr = Smalltalk::nil_object();
//     let nil_raw = RawObjectPointer::new(nil_ptr.into());
//     let nil = nil_raw.reify();
//
//     let finish = array.len();
//     let start = hash.rem_euclid(finish as u32) as usize;
//
//     fn find_item_or_empty_slot(
//         start: usize,
//         finish: usize,
//         array: &DeprecatedArray,
//         object: &AnyObject,
//         nil_object: &AnyObject,
//     ) -> Option<usize> {
//         for (index, association) in array.raw_items()[start..finish]
//             .iter()
//             .map(RawObjectPointer::reify)
//             .enumerate()
//         {
//             if association.is_identical(nil_object)? {
//                 return Some(index + 1);
//             }
//
//             let association_a = association.as_object_unchecked();
//             let key = association_a.inst_var_at(0).unwrap();
//
//             if key.is_identical(object)? {
//                 return Some(index + 1);
//             }
//         }
//
//         for (index, association) in array.raw_items()[0..start]
//             .iter()
//             .map(RawObjectPointer::reify)
//             .enumerate()
//         {
//             if association.is_identical(nil_object)? {
//                 return Some(index + 1);
//             }
//
//             let association = association.as_object_unchecked();
//             let key = association.inst_var_at(0).unwrap();
//
//             if key.is_identical(object)? {
//                 return Some(index + 1);
//             }
//         }
//         Some(0)
//     }
//
//     match find_item_or_empty_slot(start, finish, &array, &object, &nil) {
//         None => Smalltalk::primitive_fail_code(1),
//         Some(index) => Smalltalk::method_return_integer(index as i64),
//     }
// }

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveDebugPrintArray() {
    let value_ptr = Smalltalk::stack_ref(StackOffset::new(0));

    let array: &Array = &ArrayRef::try_from(value_ptr).unwrap();

    for each in array.iter() {
        if each.is_immediate() {
            println!("{:?}", each.as_immediate().unwrap());
        } else {
            println!("{:?}", each.as_object().unwrap().deref());
        }
    }

    // match ArrayRef::try_from(value_ptr)  {
    //     Ok(array_ref) => {
    //         println!("{:?}",&array_ref);
    //         Smalltalk::method_return_boolean(true);
    //     }
    //     Err(error) => {
    //         error!("{}", error);
    //         Smalltalk::primitive_fail();
    //     }
    // }
    Smalltalk::method_return_boolean(true);
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveIsOldObject() {
    let object = Smalltalk::stack_object_value(StackOffset::new(0)).unwrap();
    Smalltalk::method_return_boolean(Smalltalk::is_old(object));
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveIsYoungObject() {
    let object = Smalltalk::stack_object_value(StackOffset::new(0)).unwrap();
    Smalltalk::method_return_boolean(Smalltalk::is_young(object));
}
