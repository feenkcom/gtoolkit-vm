use crate::EventLoopMessage;
use anyhow::Result;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread::JoinHandle;
use vm_bindings::{
    InterpreterParameters, InterpreterProxy, LogLevel, NamedPrimitive, ObjectFieldIndex,
    PharoInterpreter,
};

#[no_mangle]
pub static mut VIRTUAL_MACHINE: Option<Arc<VirtualMachine>> = None;
fn vm() -> &'static Arc<VirtualMachine> {
    unsafe { VIRTUAL_MACHINE.as_ref().expect("VM must be initialized") }
}

#[derive(Debug)]
pub struct VirtualMachine {
    interpreter: Arc<PharoInterpreter>,
    event_loop_sender: Sender<EventLoopMessage>,
}

impl VirtualMachine {
    pub fn new(
        parameters: InterpreterParameters,
        event_loop_sender: Sender<EventLoopMessage>,
    ) -> Self {
        let vm = Self {
            interpreter: Arc::new(PharoInterpreter::new(parameters)),
            event_loop_sender,
        };

        vm.add_primitive(primitive!(is_virtual_machine));
        vm.add_primitive(primitive!(get_named_primitives));
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

    /// Starts the interpreter in a worker thread
    pub fn start_in_worker(&self) -> Result<JoinHandle<Result<()>>> {
        self.interpreter.clone().start_in_worker()
    }

    /// Starts the interpreter in a worker thread
    pub fn start(&self) -> Result<()> {
        self.interpreter.clone().start()
    }

    pub fn register(self: Arc<Self>) {
        unsafe { VIRTUAL_MACHINE = Some(self) };
    }
}

#[no_mangle]
pub fn is_virtual_machine() {
    vm().proxy().method_return_boolean(true);
}

#[no_mangle]
pub fn get_named_primitives() {
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
