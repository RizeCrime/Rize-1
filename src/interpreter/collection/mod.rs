use std::sync::Arc;

use bevy::prelude::*;

mod azm;
pub use azm::*;

use super::InterpreterRes;

pub fn init_interpreters(mut r_interpreters: ResMut<InterpreterRes>) {
    // The Custom Rize-1 ISA
    r_interpreters.all.insert(
        "Rize-1 Interpreter".to_string(),
        Arc::new(AzmInterpreter {}),
    );
}
