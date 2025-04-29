#![allow(dead_code, unused_variables, unused_mut)] // Heavily in-Development at the Moment

use bevy::prelude::*;

use crate::{
    types::{ActiveProgram, Registers},
    CpuCycleStage,
};

use super::InterpreterRes;

pub fn fetch(
    mut r_active_program: ResMut<ActiveProgram>,
    mut registers: ResMut<Registers>,
    mut sn_cpu: ResMut<NextState<CpuCycleStage>>,
    interpreters: Res<InterpreterRes>,
) {
    let interpreter = interpreters.active.as_ref().unwrap();
    // if None, assume EOF and Halt
    if interpreter
        .fetch(&mut registers, &mut r_active_program)
        .is_none()
    {
        sn_cpu.set(CpuCycleStage::Halt);
    }
}

pub fn decode(
    mut r_active_program: ResMut<ActiveProgram>,
    interpreters: Res<InterpreterRes>,
) {
    let interpreter = interpreters.active.as_ref().unwrap();
}
