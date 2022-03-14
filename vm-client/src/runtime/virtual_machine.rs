use crate::{
    log_signal, primitiveEnableLogSignal, primitiveEventLoopCallout, primitiveExtractReturnValue,
    primitiveGetEnabledLogSignals, primitivePollLogger, primitiveStartBeacon, primitiveStartConsoleLogger, primitiveStopLogger,
    should_log_signal, EventLoop, EventLoopCallout, EventLoopMessage,
};
use anyhow::Result;
use libffi::high::call;
use libffi::low::{ffi_cif, ffi_type, CodePtr};
use libffi::middle::Cif;
use std::mem::{size_of, transmute};
use std::os::raw::c_void;
use std::process::exit;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;
use vm_bindings::{
    InterpreterParameters, InterpreterProxy, LogLevel, NamedPrimitive, ObjectFieldIndex,
    PharoInterpreter, StackOffset,
};

use num_traits::FromPrimitive;

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
}

impl VirtualMachine {
    /// Create a Virtual Machine for given interpreter parameters.
    /// If event loop sender is `None` - start the virtual machine
    /// in the main thread rather than the worker thread
    pub fn new(
        parameters: InterpreterParameters,
        event_loop: Option<EventLoop>,
        event_loop_sender: Option<Sender<EventLoopMessage>>,
    ) -> Self {
        let vm = Self {
            interpreter: Arc::new(PharoInterpreter::new(parameters)),
            event_loop,
            event_loop_sender,
        };

        vm.interpreter().set_logger(Some(log_signal));
        vm.interpreter().set_should_log(Some(should_log_signal));

        vm.add_primitive(primitive!(primitiveGetNamedPrimitives));
        vm.add_primitive(primitive!(primitiveEventLoopCallout));
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
        if self.event_loop_sender.is_some() {
            Ok(Some(self.interpreter.clone().start_in_worker()?))
        } else {
            self.interpreter.clone().start()?;
            Ok(None)
        }
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
        } else {
            match message {
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
