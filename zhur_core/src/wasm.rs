use zhur_common::msg::chan::Envelope;
use zhur_invk::Invocation;
/// Executor module.
pub mod executor;
/// Wasm executor pool.
pub mod pool;

pub type InvocEnv = Envelope<Invocation, Vec<u8>>;
pub type PayloadEnv = Envelope<Vec<u8>, Vec<u8>>;
