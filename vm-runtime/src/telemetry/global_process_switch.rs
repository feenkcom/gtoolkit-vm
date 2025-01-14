use vm_object_model::{AnyObject, Immediate, RawObjectPointer};
use crate::{AbstractTelemetry, TelemetrySignal};

#[derive(Debug)]
pub struct GlobalProcessSwitchTelemetry {
    telemetry: RawObjectPointer,
}

impl GlobalProcessSwitchTelemetry {
    pub fn new(telemetry: RawObjectPointer) -> Self {
        Self { telemetry }
    }
}
//
// impl AbstractTelemetry for GlobalProcessSwitchTelemetry {
//     fn receive_signal(&mut self, signal: &TelemetrySignal) {
//         match signal {
//             TelemetrySignal::ContextSwitch(signal) => {
//                 match TelemetryObject::try_from(&mut self.telemetry) {
//                     Ok(mut object) => object.receive_context_switch_signal(signal),
//                     Err(error) => {
//                         error!("Failed to receive signal: {}", error);
//                     }
//                 }
//             }
//             TelemetrySignal::SemaphoreWait(signal) => {
//                 match TelemetryObject::try_from(&mut self.telemetry) {
//                     Ok(mut object) => object.receive_semaphore_wait_signal(signal),
//                     Err(error) => {
//                         error!("Failed to receive signal: {}", error);
//                     }
//                 }
//             }
//         }
//     }
//
//     fn assign_id(&mut self, id: usize) {
//         if let Some(telemetry) = self.telemetry.reify_mut().try_into_object_mut() {
//             telemetry.inst_var_at_put(0, &AnyObject::Immediate(Immediate::new_integer(id as i64)));
//         }
//     }
// }
