#![allow(dead_code, unused_variables, unused_mut)] // Heavily in-Development at the Moment

use bevy::prelude::*;

use crate::{
    display::DisplayMemory,
    types::{ActiveProgram, Registers, SystemMemory},
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
    mut memory: ResMut<SystemMemory>,
    mut registers: ResMut<Registers>,
    mut program: ResMut<ActiveProgram>,
    mut sn_cpu: ResMut<NextState<CpuCycleStage>>,
    interpreters: Res<InterpreterRes>,
) {
    let interpreter = interpreters.active.as_ref().unwrap();
    if let Err(e) =
        interpreter.decode(&mut program, &mut registers, &mut memory)
    {
        error!("Decoding Error ({:?})", e.type_,);
        sn_cpu.set(CpuCycleStage::Halt);
    }
}

pub fn execute(
    interpreters: Res<InterpreterRes>,
    mut registers: ResMut<Registers>,
    mut program: ResMut<ActiveProgram>,
    mut memory: ResMut<SystemMemory>,
    mut display_memory: ResMut<DisplayMemory>,
    images: Res<Assets<Image>>,
    mut sn_cpu: ResMut<NextState<CpuCycleStage>>,
    mut next_cpu_stage: ResMut<NextState<CpuCycleStage>>,
) {
    let interpreter = interpreters.active.as_ref().unwrap();
    // if None, assume Execution Error and Halt
    if interpreter
        .execute(&mut program, &mut registers, &mut memory, &mut display_memory, &images, next_cpu_stage)
        .is_none()
    {
        sn_cpu.set(CpuCycleStage::Halt);
    }
}
