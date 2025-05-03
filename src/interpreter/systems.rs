#![allow(dead_code, unused_variables, unused_mut)] // Heavily in-Development at the Moment

use bevy::prelude::*;

use crate::{
    types::{ActiveProgram, Registers, SystemMemory}, 
    CpuCycleStage,
    display::DisplayMemory,
};

use super::InterpreterRes;

pub fn fetch(
    mut r_active_program: ResMut<ActiveProgram>,
    mut registers: ResMut<Registers>,
    mut sn_cpu: ResMut<NextState<CpuCycleStage>>,
    interpreters: Res<InterpreterRes>,
) {
    let interpreter = interpreters.active.as_ref().unwrap();
    if interpreter
        .fetch(&mut registers, &mut r_active_program)
        .is_none()
    {
        // Fetching errors won't let Decoding stage reading the actual line to 
        //   decode (which results in ghosting CpuCycle)
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
    if let Err(e) = interpreter.decode(&mut program, &mut registers, &mut memory) {
        error!("Decoding Error: {:?}", e.type_,);
        // Decoding errors will compromise the Execute stage.
        //   For example, by not validating the arguments,
        //   Execute stage gets unexpected behavior (e.g. ignored instructions)
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
    if interpreter
        .execute(
            &mut program,
            &mut registers,
            &mut memory,
            &mut display_memory,
            &images,
        )
        .is_none()
    {
        // Halting prevents from ignoring instructions, that are incorrect/causing unexpected behavior
        // Clearly indicating that an error has occured is considerably better than unexpected behavior.
        sn_cpu.set(CpuCycleStage::Halt);
    }
}