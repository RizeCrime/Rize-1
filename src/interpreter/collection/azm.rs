#[allow(unused_imports)]
use bevy::prelude::*;

use super::super::Interpreter;

#[derive(Debug, Default)]
pub struct AzmInterpreter;

impl Interpreter for AzmInterpreter {
    fn fetch() {}
    fn decode() {}
    fn execute() {}
}
