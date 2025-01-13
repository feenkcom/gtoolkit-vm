use crate::{AbstractTelemetry, ContextSwitchSignal, TelemetrySignal};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use vm_bindings::{ObjectPointer, Smalltalk};
use vm_object_model::objects::OrderedCollectionMut;
use vm_object_model::{AnyObject, Immediate, RawObjectPointer};

#[derive(Debug)]
pub struct ProcessSwitchTelemetry {
    telemetry: RawObjectPointer,
}

impl ProcessSwitchTelemetry {
    pub fn new(telemetry: RawObjectPointer) -> Self {
        Self { telemetry }
    }
}

impl AbstractTelemetry for ProcessSwitchTelemetry {
    fn receive_signal(&mut self, signal: &TelemetrySignal) {
        match signal {
            TelemetrySignal::ContextSwitch(signal) => {
                match TelemetryObject::try_from(&mut self.telemetry) {
                    Ok(mut object) => object.receive_signal(signal),
                    Err(error) => {
                        error!("Failed to receive signal: {}", error);
                    }
                }
            }
        }
    }

    fn assign_id(&mut self, id: usize) {
        if let Some(telemetry) = self.telemetry.reify_mut().try_into_object_mut() {
            telemetry.inst_var_at_put(0, &AnyObject::Immediate(Immediate::new_integer(id as i64)));
        }
    }
}

struct TelemetryObject<'image> {
    signals: OrderedCollectionMut<'image>,
    current_process: RawObjectPointer,
    signal_class: RawObjectPointer,
}

impl<'image> TelemetryObject<'image> {
    fn receive_signal(&'image mut self, signal: &ContextSwitchSignal) {
        println!("[receive_signal] {:?}", signal);

        if signal.old_process == self.current_process {
            // switches away
            self.add_context_switch_signal(false);
        } else if signal.new_process == self.current_process {
            // switches back
            self.add_context_switch_signal(true);
        }
    }

    fn add_context_switch_signal(&'image mut self, alive: bool) {
        let signal_pointer = Smalltalk::instantiate_class(
            ObjectPointer::from(self.signal_class.as_i64()),
            alive,
        );
        let mut signal_pointer = RawObjectPointer::new(signal_pointer.as_i64());

        let signal_object = signal_pointer
            .reify_mut()
            .into_object_unchecked_mut();


        let since_the_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH).unwrap();

        signal_object.inst_var_at_put(
            0,
            &AnyObject::Immediate(Immediate::new_integer(since_the_epoch.as_secs() as i64)),
        );

        signal_object.inst_var_at_put(
            1,
            &AnyObject::Immediate(Immediate::new_integer(since_the_epoch.subsec_nanos() as i64)),
        );

        signal_object.inst_var_at_put(
            2,
            &RawObjectPointer::new(Smalltalk::bool_object(alive).as_i64()).reify(),
        );

        self.signals.add_last(&AnyObject::Object(signal_object));
    }
}

impl<'image> TryFrom<&'image mut RawObjectPointer> for TelemetryObject<'image> {
    type Error = String;

    fn try_from(pointer: &'image mut RawObjectPointer) -> Result<Self, Self::Error> {
        let mut object = pointer.reify_mut();

        let telemetry_object = object
            .try_into_object_mut()
            .ok_or_else(|| "Expected an object".to_string())?;

        let current_process = telemetry_object
            .inst_var_at(2)
            .ok_or_else(|| "Telemetry should have `currentProcess` inst.var".to_string())?
            .raw_header();

        let signal_class = telemetry_object
            .inst_var_at(3)
            .ok_or_else(|| "Telemetry should have `signalClass` inst.var".to_string())?
            .raw_header();

        let signals_object = telemetry_object
            .inst_var_at_mut(1)
            .ok_or_else(|| "Telemetry should have `signals` inst.var".to_string())?;
        let signals = OrderedCollectionMut::try_from(signals_object)?;

        Ok(Self {
            signals,
            current_process,
            signal_class,
        })
    }
}
