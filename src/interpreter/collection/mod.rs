use std::sync::Arc;

use bevy::prelude::*;

#[cfg(test)]
mod tests;

mod azm;
mod opcode_fn;
pub use azm::*;

use super::InterpreterRes;

pub fn init_interpreters(mut r_interpreters: ResMut<InterpreterRes>) {
    // The Custom Rize-1 ISA
    r_interpreters.all.insert(
        "Rize-1 Interpreter".to_string(),
        Arc::new(AzmInterpreter {}),
    );
    r_interpreters.active = Some(
        r_interpreters
            .all
            .get("Rize-1 Interpreter")
            .unwrap()
            .clone(),
    );
}
