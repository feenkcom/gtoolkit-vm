mod base_logger;
mod beacon_logger;

pub use base_logger::{
    log_signal, primitiveEnableLogSignal, primitiveGetEnabledLogSignals, primitivePollLogger,
    primitiveStopLogger, should_log_signal, LogSignal, Logger, NullLogger, VM_LOGGER,
};
pub use beacon_logger::primitiveStartBeacon;
