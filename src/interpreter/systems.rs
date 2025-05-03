#![allow(dead_code, unused_variables, unused_mut)] // Heavily in-Development at the Moment

use bevy::prelude::*;

use crate::{
    display::DisplayMemory,
    types::{ActiveProgram, ProgramSettings, Registers, SystemMemory},
    CpuCycleStage,
};

use super::InterpreterRes;

pub fn auto_step(
    mut program: ResMut<ActiveProgram>,
    settings: Res<ProgramSettings>,
    mut registers: ResMut<Registers>,
    mut memory: ResMut<SystemMemory>,
    mut display_memory: ResMut<DisplayMemory>,
    mut sn_cpu: ResMut<NextState<CpuCycleStage>>,
    interpreters: Res<InterpreterRes>,
) {
    for _ in 0..settings.autostep_lines {
        unsafe {
            fetch(
                std::mem::transmute_copy(&program),
                std::mem::transmute_copy(&registers),
                std::mem::transmute_copy(&sn_cpu),
                std::mem::transmute_copy(&interpreters),
            );
            decode(
                std::mem::transmute_copy(&memory),
                std::mem::transmute_copy(&registers),
                std::mem::transmute_copy(&program),
                std::mem::transmute_copy(&sn_cpu),
                std::mem::transmute_copy(&interpreters),
            );
            execute(
                std::mem::transmute_copy(&interpreters),
                std::mem::transmute_copy(&registers),
                std::mem::transmute_copy(&program),
                std::mem::transmute_copy(&memory),
                std::mem::transmute_copy(&display_memory),
                std::mem::transmute_copy(&sn_cpu),
            );
        }
    }
}

pub fn setup(
    mut registers: ResMut<Registers>,
    interpreters: Res<InterpreterRes>,
) {
    let interpreter = interpreters.active.as_ref().unwrap();
    interpreter.setup_registers(&mut registers);
}

pub fn fetch(
    mut program: ResMut<ActiveProgram>,
    mut registers: ResMut<Registers>,
    mut sn_cpu: ResMut<NextState<CpuCycleStage>>,
    interpreters: Res<InterpreterRes>,
) {
    let interpreter = interpreters.active.as_ref().unwrap();
    if interpreter.fetch(&mut registers, &mut program).is_none() {
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
    if let Err(e) =
        interpreter.decode(&mut program, &mut registers, &mut memory)
    {
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
    mut sn_cpu: ResMut<NextState<CpuCycleStage>>,
) {
    let interpreter = interpreters.active.as_ref().unwrap();
    // The Execute Stage needs Access to sn_cpu anyway,
    // so it'll have to Halt itself
    let _ = interpreter.execute(
        &mut program,
        &mut registers,
        &mut memory,
        &mut display_memory,
        sn_cpu,
    );
}
