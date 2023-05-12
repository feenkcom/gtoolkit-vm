use std::cell::RefCell;
use std::ffi::{c_int, CString};
use std::mem::transmute;
use std::os::raw::c_void;
use std::process::exit;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread::JoinHandle;

use anyhow::Result;

use vm_bindings::{
    virtual_machine_info, InterpreterConfiguration, InterpreterProxy, LogLevel, NamedPrimitive,
    ObjectFieldIndex, PharoInterpreter, StackOffset,
};

use crate::version::{app_info, app_version};
use crate::{
    log_signal, primitiveEnableLogSignal, primitiveGetEnabledLogSignals, primitivePollLogger,
    primitiveStartBeacon, primitiveStartConsoleLogger, primitiveStopLogger, should_log_all_signals,
    should_log_signal, ConsoleLogger, EventLoop, EventLoopMessage, EventLoopWaker, VM_LOGGER,
};
#[cfg(feature = "ffi")]
use crate::{primitiveEventLoopCallout, primitiveExtractReturnValue};

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

        vm.add_primitive(primitive!(primitiveGetNamedPrimitives));
        #[cfg(feature = "ffi")]
        vm.add_primitive(primitive!(primitiveEventLoopCallout));
        #[cfg(feature = "ffi")]
        vm.add_primitive(primitive!(primitiveExtractReturnValue));
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
    vm().proxy().method_return_boolean(true);
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveGetNamedPrimitives() {
    let proxy = vm().proxy();
    let named_primitives = vm().named_primitives();

    let return_array =
        proxy.instantiate_indexable_class_of_size(proxy.class_array(), named_primitives.len());
    for (index, named_primitive) in named_primitives.iter().enumerate() {
        let each_primitive_array =
            proxy.instantiate_indexable_class_of_size(proxy.class_array(), 3);

        let plugin_name = proxy.new_string(named_primitive.plugin_name());
        let primitive_name = proxy.new_string(named_primitive.primitive_name());
        let primitive_address = proxy.new_external_address(named_primitive.primitive_address());

        proxy.item_at_put(each_primitive_array, ObjectFieldIndex::new(1), plugin_name);
        proxy.item_at_put(
            each_primitive_array,
            ObjectFieldIndex::new(2),
            primitive_name,
        );
        proxy.item_at_put(
            each_primitive_array,
            ObjectFieldIndex::new(3),
            primitive_address,
        );

        proxy.item_at_put(
            return_array,
            ObjectFieldIndex::new(index + 1),
            each_primitive_array,
        );
    }
    proxy.method_return_value(return_array);
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveGetSemaphoreSignaller() {
    let proxy = vm().proxy();
    let signaller = proxy.new_external_address(semaphore_signaller as *const c_void);
    proxy.method_return_value(signaller);
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveGetEventLoop() {
    let vm = vm();
    let proxy = vm.proxy();

    let event_loop = match vm.event_loop() {
        None => std::ptr::null(),
        Some(event_loop) => event_loop as *const EventLoop,
    };

    proxy.method_return_value(proxy.new_external_address(event_loop));
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveGetEventLoopReceiver() {
    let proxy = vm().proxy();
    let receiver = proxy.new_external_address(try_receive_events as *const c_void);
    proxy.method_return_value(receiver);
}

#[no_mangle]
#[allow(non_snake_case)]
#[cfg(target_os = "android")]
/// Returns a reference to a clone of the AndroidApp.
/// Users are responsible for managing the memory
pub fn primitiveGetAndroidApp() {
    let vm = vm();
    let proxy = vm.proxy();

    let android_app = Box::into_raw(Box::new(vm.android_app.clone()));
    proxy.method_return_value(proxy.new_external_address(android_app));
}

#[no_mangle]
#[allow(non_snake_case)]
#[cfg(target_os = "android")]
/// Returns a pointer to the NativeWindow
pub fn primitiveGetAndroidNativeWindow() {
    let vm = vm();
    let proxy = vm.proxy();

    let native_window = vm
        .android_app
        .native_window()
        .map(|native_window| native_window.ptr().as_ptr())
        .unwrap_or(std::ptr::null_mut());

    proxy.method_return_value(proxy.new_external_address(native_window));
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
    let proxy = vm().proxy();

    let waker_thunk_external_address = proxy.stack_object_value(StackOffset::new(1));
    let waker_function_external_address = proxy.stack_object_value(StackOffset::new(0));

    let waker_function_ptr = proxy.read_address(waker_function_external_address);

    let waker_function: extern "C" fn(*const c_void, u32) -> bool =
        unsafe { transmute(waker_function_ptr) };
    let waker_thunk = proxy.read_address(waker_thunk_external_address);

    let waker = EventLoopWaker::new(waker_function, waker_thunk);
    vm().event_loop_waker.replace(Some(waker));

    proxy.method_return_boolean(true);
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveFullGarbageCollectorMicroseconds() {
    let proxy = vm().proxy();

    let microseconds = vm().interpreter().full_gc_microseconds();
    proxy.method_return_value(proxy.new_positive_64bit_integer(microseconds));
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveScavengeGarbageCollectorMicroseconds() {
    let proxy = vm().proxy();

    let microseconds = vm().interpreter().scavenge_gc_microseconds();
    proxy.method_return_value(proxy.new_positive_64bit_integer(microseconds));
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveFirstBytePointerOfDataObject() {
    let proxy = vm().proxy();
    let receiver = proxy.stack_object_value(StackOffset::new(0));

    let pointer = proxy.first_byte_pointer_of_data_object(receiver);
    proxy.method_return_value(proxy.new_external_address(pointer));
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitivePointerAtPointer() {
    let proxy = vm().proxy();
    let external_address = proxy.stack_object_value(StackOffset::new(0));

    let external_address_pointer = proxy.read_address(external_address);
    let pointer = proxy.pointer_at_pointer(external_address_pointer);
    proxy.method_return_value(proxy.new_external_address(pointer));
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveVirtualMachineInfo() {
    let proxy = vm().proxy();
    let info = virtual_machine_info();
    proxy.method_return_value(proxy.new_string(info));
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveAppInfo() {
    let proxy = vm().proxy();
    let info = app_info();
    proxy.method_return_value(proxy.new_string(info));
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveAppVersion() {
    let proxy = vm().proxy();
    let version = app_version();
    proxy.method_return_value(proxy.new_string(version));
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveFcntl() {
    let proxy = vm().proxy();
    #[cfg(unix)]
    {
        match proxy.method_argument_count() {
            2 => {
                let file_descriptor = proxy.stack_integer_value(StackOffset::new(1)) as c_int;
                let command = proxy.stack_integer_value(StackOffset::new(0)) as c_int;

                let result = unsafe { libc::fcntl(file_descriptor, command) };
                proxy.method_return_value(proxy.new_integer(result));
            }
            3 => {
                let file_descriptor = proxy.stack_integer_value(StackOffset::new(2)) as c_int;
                let command = proxy.stack_integer_value(StackOffset::new(1)) as c_int;
                let argument = proxy.stack_integer_value(StackOffset::new(0)) as c_int;

                let result = unsafe { libc::fcntl(file_descriptor, command, argument) };
                proxy.method_return_value(proxy.new_integer(result));
            }
            _ => proxy.primitive_fail(),
        }
    }
    #[cfg(not(unix))]
    {
        proxy.primitive_fail();
    }
}
