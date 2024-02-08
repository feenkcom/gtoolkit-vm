use crate::vm;
use std::collections::LinkedList;
use std::ffi::{c_void, CStr, CString};
use std::time::{Duration, Instant};
use vm_bindings::{
    sqInt, InterpreterTelemetry, NativeTransmutable, ObjectFieldIndex, ObjectPointer, StackOffset,
};

include!(concat!(env!("OUT_DIR"), "/telemetry-cache.rs"));

#[derive(Debug)]
struct Telemetry {
    signals: LinkedList<TelemetrySignal>,
    signals_array: Vec<TelemetrySignal>,
    start_time: Instant,
    in_machine_code_leaf_primitive: bool,
    entering_machine_method: bool,
}

#[derive(Debug, Clone)]
#[repr(u8)]
enum TelemetrySignal {
    Send(SendSignal),
    Return(ReturnSignal),
    PrimitiveActivation(PrimitiveActivationSignal),
    ContextSwitch(ContextSwitchSignal),
    Debug(DebugSignal),
}

impl Telemetry {
    pub fn new() -> Self {
        Self {
            signals: Default::default(),
            signals_array: vec![],
            start_time: Instant::now(),
            in_machine_code_leaf_primitive: false,
            entering_machine_method: false,
        }
    }

    pub fn add_signal(&mut self, signal: TelemetrySignal) {
        self.signals.push_back(signal)
    }

    pub fn amount_of_signals(&self) -> usize {
        self.signals_array.len()
    }

    pub fn signal_at(&self, index: usize) -> Option<&TelemetrySignal> {
        self.signals_array.get(index)
    }

    pub fn receive_send_signal(
        &mut self,
        class_index: sqInt,
        selector: Option<ObjectPointer>,
        is_immediate: bool,
        source_id: u8,
        frame_pointer: *mut c_void,
    ) {
        self.add_signal(TelemetrySignal::Debug(DebugSignal::MessageSend));

        if self.entering_machine_method {
            if source_id == 1 {
                return;
            }
            self.entering_machine_method = false;
        }

        if self.in_machine_code_leaf_primitive {
            self.return_from_leaf_primitive(frame_pointer);
        }

        let selector = selector.map(|selector| PharoString::from_selector(selector));

        let class_index = if is_immediate {
            ClassIndex::Immediate(class_index)
        } else {
            ClassIndex::Class(class_index)
        };

        let signal = TelemetrySignal::Send(SendSignal {
            timestamp: Instant::now().duration_since(self.start_time),
            class_index,
            selector,
            source_id,
            frame_pointer,
        });

        self.add_signal(signal);
    }

    pub fn receive_activate_machine_method(&mut self) {
        self.entering_machine_method = true;
        self.add_signal(TelemetrySignal::Debug(DebugSignal::ActivateMachineMethod));
    }

    pub fn receive_begin_machine_method(&mut self) {
        self.entering_machine_method = false;
        self.add_signal(TelemetrySignal::Debug(DebugSignal::BeginMachineMethod));
    }

    pub fn receive_return_signal(
        &mut self,
        source_id: u8,
        execution_location: u8,
        frame_pointer: *mut c_void,
    ) {
        self.entering_machine_method = false;

        if source_id == 25 {
            self.in_machine_code_leaf_primitive = false;
        }

        if self.in_machine_code_leaf_primitive {
            self.return_from_leaf_primitive(frame_pointer);
        }

        self.add_signal(TelemetrySignal::Return(ReturnSignal {
            timestamp: Instant::now().duration_since(self.start_time),
            source_id,
            execution_location,
            frame_pointer,
        }));
    }

    pub fn receive_machine_code_primitive_activation_signal(&mut self, signal_id: u8) {
        self.in_machine_code_leaf_primitive = signal_id == 1;

        match signal_id {
            0 => self.add_signal(TelemetrySignal::Debug(
                DebugSignal::DeactivateMachinePrimitive,
            )),
            1 => self.add_signal(TelemetrySignal::Debug(
                DebugSignal::ActivateMachinePrimitive,
            )),
            2 => self.add_signal(TelemetrySignal::Debug(
                DebugSignal::MachinePrimitiveMayCallMethods,
            )),
            _ => {}
        };
    }

    pub fn receive_context_switch_signal(
        &mut self,
        old_process: ObjectPointer,
        new_process: ObjectPointer,
    ) {
        let proxy = vm().proxy();
        self.add_signal(TelemetrySignal::ContextSwitch(ContextSwitchSignal {
            timestamp: Instant::now().duration_since(self.start_time),
            old_process: proxy.nil_object(),
            new_process: proxy.nil_object(),
        }));
    }

    pub fn receive_debug_class_signal(&mut self, class_index: sqInt, is_immediate: bool) {
        let class_index = if is_immediate {
            ClassIndex::Immediate(class_index)
        } else {
            ClassIndex::Class(class_index)
        };
        self.add_signal(TelemetrySignal::Debug(DebugSignal::Class(class_index)))
    }

    pub fn receive_debug_selector_signal(&mut self, selector: Option<ObjectPointer>) {
        let selector = selector.map(|selector| PharoString::from_selector(selector));
        self.add_signal(TelemetrySignal::Debug(DebugSignal::Selector(selector)))
    }

    fn return_from_leaf_primitive(&mut self, frame_pointer: *mut c_void) {
        self.in_machine_code_leaf_primitive = false;
        self.add_signal(TelemetrySignal::Return(ReturnSignal {
            timestamp: Instant::now().duration_since(self.start_time),
            source_id: 25,
            execution_location: 2,
            frame_pointer,
        }));
    }

    pub fn timestamp_nanos(&self, duration: &Duration) -> u64 {
        duration.as_secs() * 1000000000 + duration.subsec_nanos() as u64
    }

    pub fn convert_signals_to_array(&mut self) {
        let current_signals = std::mem::replace(&mut self.signals, Default::default());
        self.signals_array = current_signals.into_iter().collect();
    }

    pub fn into_interpreter_telemetry(self) -> InterpreterTelemetry {
        InterpreterTelemetry {
            payload: Box::into_raw(Box::new(self)) as *mut c_void,
            sendFn: Some(telemetry_receive_send_signal),
            returnFn: Some(telemetry_receive_return_signal),
            primitiveActivationFn: Some(telemetry_receive_primitive_activation_signal),
            activateMachineMethodFn: Some(telemetry_receive_activate_machine_method),
            beginMachineMethodFn: Some(telemetry_receive_begin_machine_method),
            contextSwitchFn: Some(telemetry_receive_context_switch_signal),
            debugRecordClassFn: Some(telemetry_receive_debug_class_signal),
            debugRecordSelectorFn: Some(telemetry_receive_debug_selector_signal),
        }
    }
}

impl From<&TelemetrySignal> for u8 {
    fn from(signal: &TelemetrySignal) -> u8 {
        match signal {
            TelemetrySignal::Send(_) => 1,
            TelemetrySignal::Return(_) => 2,
            TelemetrySignal::PrimitiveActivation(_) => 3,
            TelemetrySignal::ContextSwitch(_) => 4,
            TelemetrySignal::Debug(_) => 5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SendSignal {
    timestamp: Duration,
    class_index: ClassIndex,
    selector: Option<PharoString>,
    source_id: u8,
    frame_pointer: *mut c_void,
}

#[derive(Debug, Clone)]
pub struct ReturnSignal {
    timestamp: Duration,
    source_id: u8,
    execution_location: u8,
    frame_pointer: *mut c_void,
}

#[derive(Debug, Clone)]
pub struct PrimitiveActivationSignal {
    signal_id: u8,
}

#[derive(Debug, Clone)]
pub struct ContextSwitchSignal {
    timestamp: Duration,
    old_process: ObjectPointer,
    new_process: ObjectPointer,
}

#[derive(Debug, Clone)]
pub enum DebugSignal {
    ActivateMachineMethod,
    BeginMachineMethod,
    ActivateMachinePrimitive,
    DeactivateMachinePrimitive,
    MachinePrimitiveMayCallMethods,
    MessageSend,
    Class(ClassIndex),
    Selector(Option<PharoString>),
}

#[derive(Debug, Clone)]
pub enum PharoString {
    Class(usize),
    Selector(usize),
    String(CString),
    Unsupported,
}

#[derive(Debug, Clone)]
pub enum ClassIndex {
    Class(sqInt),
    Immediate(sqInt),
}

impl PharoString {
    pub fn from_selector(symbol: ObjectPointer) -> Self {
        let proxy = vm().proxy();

        let pointer = proxy.first_byte_pointer_of_data_object(symbol);
        let len = proxy.size_of(symbol);
        let slice = unsafe { std::slice::from_raw_parts(pointer as *const u8, len) };
        let index: Option<&usize> = SELECTORS_MAP.get(slice);
        index
            .map(|index| PharoString::Selector(*index))
            .unwrap_or_else(|| {
                CString::new(slice)
                    .ok()
                    .map(|string| PharoString::String(string))
                    .unwrap_or(PharoString::Unsupported)
            })
    }

    pub fn as_pharo_string(&self) -> ObjectPointer {
        match self {
            PharoString::Class(index) => Self::str_to_pharo_string(CLASSES[*index]),
            PharoString::Selector(index) => Self::str_to_pharo_string(SELECTORS[*index]),
            PharoString::String(string) => Self::cstr_to_pharo_string(string.as_c_str()),
            PharoString::Unsupported => vm().proxy().nil_object(),
        }
    }

    fn str_to_pharo_string(string: &str) -> ObjectPointer {
        vm().proxy().new_string(string)
    }

    fn cstr_to_pharo_string(string: &CStr) -> ObjectPointer {
        vm().proxy().new_string_from_cstring(Some(string))
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveStartTelemetry() {
    let interpreter = vm().interpreter();
    let proxy = interpreter.proxy();

    // get rid of the previous telemetry if any
    {
        let interpreter_telemetry = interpreter.take_telemetry();
        if let Some(mut interpreter_telemetry) = interpreter_telemetry {
            let telemetry = interpreter_telemetry.payload as *mut Telemetry;
            interpreter_telemetry.payload = std::ptr::null_mut();
            unsafe {
                let _ = Box::from_raw(telemetry);
            };
        }
    }

    let telemetry = Telemetry::new();
    interpreter.set_telemetry(telemetry.into_interpreter_telemetry());
    interpreter.enable_telemetry();
    proxy.method_return_boolean(true);
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveStopTelemetry() {
    let interpreter = vm().interpreter();
    let proxy = interpreter.proxy();

    let interpreter_telemetry = interpreter.take_telemetry();
    let telemetry_address = if let Some(mut interpreter_telemetry) = interpreter_telemetry {
        let telemetry = interpreter_telemetry.payload as *mut Telemetry;
        interpreter_telemetry.payload = std::ptr::null_mut();
        telemetry
    } else {
        std::ptr::null_mut()
    };

    if !telemetry_address.is_null() {
        let mut telemetry = unsafe { Box::from_raw(telemetry_address) };
        telemetry.convert_signals_to_array();
        Box::leak(telemetry);
    }

    proxy.method_return_value(proxy.new_external_address(telemetry_address));
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveDisableTelemetry() {
    let interpreter = vm().interpreter();
    let proxy = interpreter.proxy();

    interpreter.disable_telemetry();

    proxy.method_return_boolean(true);
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveAmountOfTelemetrySignals() {
    let interpreter = vm().interpreter();
    let proxy = interpreter.proxy();

    let telemetry_oop = proxy.stack_object_value(StackOffset::new(0));
    if !proxy.is_kind_of_class(telemetry_oop, proxy.class_external_address()) {
        // telemetry address must be stored in a an ExternalAddress
        return proxy.primitive_fail();
    }

    let telemetry_address = proxy.read_address(telemetry_oop) as *mut Telemetry;
    if telemetry_address.is_null() {
        return proxy.method_return_integer(0);
    }

    let telemetry = unsafe { Box::from_raw(telemetry_address) };
    let amount = telemetry.amount_of_signals();
    Box::leak(telemetry);
    proxy.method_return_integer(amount as i64);
}

/// Arguments: array of classes, telemetry address, signal index
///
/// Expects an Array with classes for each telemetry signal:
/// [ SendSignal, ReturnSignal, ContextSwitchSignal ]
#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveReifyTelemetrySignalAt() {
    let interpreter = vm().interpreter();
    let proxy = interpreter.proxy();

    let classes = proxy.stack_object_value(StackOffset::new(2));
    if !proxy.is_kind_of_class(classes, proxy.class_array()) {
        error!("Signal types must be an Array");
        // classes must be stored in a an array
        return proxy.primitive_fail();
    }
    let classes_len = proxy.size_of(classes);

    let telemetry_oop = proxy.stack_object_value(StackOffset::new(1));
    if !proxy.is_kind_of_class(telemetry_oop, proxy.class_external_address()) {
        error!("Telemetry handle must be an ExternalAddress");
        // telemetry address must be stored in a an ExternalAddress
        return proxy.primitive_fail();
    }
    let telemetry_address = proxy.read_address(telemetry_oop) as *mut Telemetry;
    if telemetry_address.is_null() {
        return proxy.method_return_value(proxy.nil_object());
    }

    let signal_index = proxy.stack_integer_value(StackOffset::new(0)) as usize;

    let telemetry = unsafe { Box::from_raw(telemetry_address) };
    let signal_object = if let Some(signal) = telemetry.signal_at(signal_index - 1) {
        let signal_type = Into::<u8>::into(signal) as usize;
        if signal_type > classes_len {
            error!(
                "Signal class wasn't provided for signal type {}",
                signal_type
            );
            /// Didn't provide all signal types
            return proxy.primitive_fail();
        } else {
            let signal_oop = proxy.instantiate_class(
                proxy.item_at(classes, ObjectFieldIndex::new(signal_type)),
                false,
            );

            match signal {
                TelemetrySignal::Send(send_signal) => {
                    proxy.object_field_at_put(
                        signal_oop,
                        ObjectFieldIndex::new(0),
                        proxy.new_positive_64bit_integer(
                            telemetry.timestamp_nanos(&send_signal.timestamp),
                        ),
                    );
                    proxy.object_field_at_put(
                        signal_oop,
                        ObjectFieldIndex::new(1),
                        proxy.new_external_address(send_signal.frame_pointer),
                    );

                    proxy.object_field_at_put(
                        signal_oop,
                        ObjectFieldIndex::new(2),
                        match send_signal.class_index {
                            ClassIndex::Class(index) => proxy.class_or_nil_at_index(index),
                            ClassIndex::Immediate(value) => ObjectPointer::from_native_c(value),
                        },
                    );

                    if let Some(ref selector) = send_signal.selector {
                        proxy.object_field_at_put(
                            signal_oop,
                            ObjectFieldIndex::new(3),
                            selector.as_pharo_string(),
                        );
                    }
                    proxy.object_field_at_put(
                        signal_oop,
                        ObjectFieldIndex::new(4),
                        proxy.new_integer(send_signal.source_id),
                    );
                }
                TelemetrySignal::Return(return_signal) => {
                    proxy.object_field_at_put(
                        signal_oop,
                        ObjectFieldIndex::new(0),
                        proxy.new_positive_64bit_integer(
                            telemetry.timestamp_nanos(&return_signal.timestamp),
                        ),
                    );
                    proxy.object_field_at_put(
                        signal_oop,
                        ObjectFieldIndex::new(1),
                        proxy.new_external_address(return_signal.frame_pointer),
                    );
                    proxy.object_field_at_put(
                        signal_oop,
                        ObjectFieldIndex::new(2),
                        proxy.new_integer(return_signal.source_id),
                    );
                    proxy.object_field_at_put(
                        signal_oop,
                        ObjectFieldIndex::new(3),
                        proxy.new_integer(return_signal.execution_location),
                    );
                }
                TelemetrySignal::PrimitiveActivation(primitive_activation_signal) => {
                    proxy.object_field_at_put(
                        signal_oop,
                        ObjectFieldIndex::new(0),
                        proxy.new_integer(primitive_activation_signal.signal_id),
                    );
                }
                TelemetrySignal::ContextSwitch(_) => {}
                TelemetrySignal::Debug(debug_signal) => {
                    proxy.object_field_at_put(
                        signal_oop,
                        ObjectFieldIndex::new(0),
                        proxy.new_string(format!("{:?}", debug_signal)),
                    );

                    match debug_signal {
                        DebugSignal::Class(class_index) => {
                            proxy.object_field_at_put(
                                signal_oop,
                                ObjectFieldIndex::new(1),
                                match class_index {
                                    ClassIndex::Class(index) => proxy.class_or_nil_at_index(*index),
                                    ClassIndex::Immediate(value) => {
                                        ObjectPointer::from_native_c(*value)
                                    }
                                },
                            );
                        },
                        DebugSignal::Selector(selector) => {
                            if let Some(ref selector) = selector {
                                proxy.object_field_at_put(
                                    signal_oop,
                                    ObjectFieldIndex::new(2),
                                    selector.as_pharo_string(),
                                );
                            }
                        },
                        _ => {}
                    }
                }
            }

            signal_oop
        }
    } else {
        error!(
            "Signal at index {} wasn't found. Total amount of signals: {}",
            signal_index,
            telemetry.amount_of_signals()
        );
        proxy.nil_object()
    };

    Box::leak(telemetry);

    proxy.method_return_value(signal_object);
}

#[no_mangle]
#[allow(non_snake_case)]
pub fn primitiveDropTelemetry() {
    let interpreter = vm().interpreter();
    let proxy = interpreter.proxy();

    let telemetry_oop = proxy.stack_object_value(StackOffset::new(0));
    if !proxy.is_kind_of_class(telemetry_oop, proxy.class_external_address()) {
        // telemetry address must be stored in a an ExternalAddress
        return proxy.primitive_fail();
    }

    let telemetry_address = proxy.read_address(telemetry_oop) as *mut Telemetry;
    if telemetry_address.is_null() {
        return proxy.method_return_boolean(false);
    }

    // drop telemetry
    {
        let _ = unsafe { Box::from_raw(telemetry_address) };
    };
    proxy.method_return_boolean(true)
}

#[no_mangle]
pub unsafe extern "C" fn telemetry_receive_send_signal(
    telemetry: *mut c_void,
    class_index: sqInt,
    selector: sqInt,
    is_immediate: u8,
    source_id: u8,
    frame_pointer: *mut c_void,
) {
    let mut telemetry = Box::from_raw(telemetry as *mut Telemetry);
    let selector = if selector == 0 {
        None
    } else {
        Some(ObjectPointer::from_native_c(selector))
    };

    telemetry.receive_send_signal(
        class_index,
        selector,
        is_immediate != 0,
        source_id,
        frame_pointer,
    );
    Box::leak(telemetry);
}

#[no_mangle]
pub unsafe extern "C" fn telemetry_receive_activate_machine_method(telemetry: *mut c_void) {
    let mut telemetry = Box::from_raw(telemetry as *mut Telemetry);
    telemetry.receive_activate_machine_method();
    Box::leak(telemetry);
}

#[no_mangle]
pub unsafe extern "C" fn telemetry_receive_begin_machine_method(telemetry: *mut c_void) {
    let mut telemetry = Box::from_raw(telemetry as *mut Telemetry);
    telemetry.receive_begin_machine_method();
    Box::leak(telemetry);
}

#[no_mangle]
pub unsafe extern "C" fn telemetry_receive_return_signal(
    telemetry: *mut c_void,
    source_id: u8,
    execution_location: u8,
    frame_pointer: *mut c_void,
) {
    let mut telemetry = Box::from_raw(telemetry as *mut Telemetry);

    telemetry.receive_return_signal(source_id, execution_location, frame_pointer);
    Box::leak(telemetry);
}

#[no_mangle]
pub unsafe extern "C" fn telemetry_receive_primitive_activation_signal(
    telemetry: *mut c_void,
    signal_id: u8,
) {
    let mut telemetry = Box::from_raw(telemetry as *mut Telemetry);
    telemetry.receive_machine_code_primitive_activation_signal(signal_id);
    Box::leak(telemetry);
}

#[no_mangle]
pub unsafe extern "C" fn telemetry_receive_context_switch_signal(
    telemetry: *mut c_void,
    old_process: sqInt,
    new_process: sqInt,
) {
    let mut telemetry = Box::from_raw(telemetry as *mut Telemetry);
    telemetry.receive_context_switch_signal(
        ObjectPointer::from_native_c(old_process),
        ObjectPointer::from_native_c(new_process),
    );
    Box::leak(telemetry);
}

#[no_mangle]
pub unsafe extern "C" fn telemetry_receive_debug_class_signal(
    telemetry: *mut c_void,
    class_index: sqInt,
    is_immediate: u8,
) {
    let mut telemetry = Box::from_raw(telemetry as *mut Telemetry);

    telemetry.receive_debug_class_signal(class_index, is_immediate != 0);
    Box::leak(telemetry);
}

#[no_mangle]
pub unsafe extern "C" fn telemetry_receive_debug_selector_signal(
    telemetry: *mut c_void,
    selector: sqInt,
) {
    let mut telemetry = Box::from_raw(telemetry as *mut Telemetry);
    let selector = if selector == 0 {
        None
    } else {
        Some(ObjectPointer::from_native_c(selector))
    };

    telemetry.receive_debug_selector_signal(selector);
    Box::leak(telemetry);
}
