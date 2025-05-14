use std::fs;

use bevy::prelude::*;

use crate::{
    constants::{
        FLAG_CARRY, FLAG_NEGATIVE, FLAG_OVERFLOW, FLAG_ZERO,
        N_GENERAL_PURPOSE_REGISTERS, PROGRAM_COUNTER,
    },
    interpreter::InterpreterRes,
    types::{ActiveProgram, AzmPrograms, FileCheckTimer, Register, Registers},
    ChunkSize,
};

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[allow(dead_code)] // may need it again later?
pub fn setup_registers(mut r_registers: ResMut<Registers>) {
    info!("Setting up Basic Registers...");

    let program_counter = Register::normal(PROGRAM_COUNTER);
    let memory_address_register = Register::normal("mar");
    let memory_data_register = Register::normal("mdr");

    r_registers.insert(program_counter);
    r_registers.insert(memory_address_register);
    r_registers.insert(memory_data_register);

    info!("Finished setting up Basic Registers.");

    info!("Setting up Flags...");

    let zero_flag = Register::flag(FLAG_ZERO);
    let carry_flag = Register::flag(FLAG_CARRY);
    let overflow_flag = Register::flag(FLAG_OVERFLOW);
    let negative_flag = Register::flag(FLAG_NEGATIVE);

    r_registers.insert(zero_flag);
    r_registers.insert(carry_flag);
    r_registers.insert(overflow_flag);
    r_registers.insert(negative_flag);

    info!("Finished setting up Flags.");
    info!("Setting up General Purpose Registers...");

    for i in 0..N_GENERAL_PURPOSE_REGISTERS {
        // Convert index to letter (0->a, 1->b, etc)
        let letter = (b'a' + i as u8) as char;
        let reg_name = format!("g{}", letter);
        let gpr = Register::normal(reg_name);
        r_registers.as_mut().insert(gpr);
    }

    info!("Finished setting up General Purpose Registers.");
}

pub fn check_programs(
    mut r_programs: ResMut<AzmPrograms>,
    time: Res<Time>,
    mut timer: ResMut<FileCheckTimer>,
    r_interpreters: Res<InterpreterRes>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let interpreter = if let Some(interpreter) = r_interpreters.active.as_ref()
    {
        interpreter
    } else {
        return;
    };

    let program_dir = "programs/";
    let file_extension = interpreter.file_type();

    let entries = match fs::read_dir(program_dir) {
        Ok(entries) => entries,
        Err(e) => {
            error!("Error reading directory: {}", e);
            return;
        }
    };

    entries.for_each(|entry| {
        if entry.is_err() {
            return;
        }
        let entry = entry.unwrap();

        if !entry.path().is_file() {
            return;
        }

        if r_programs.0.iter().any(|x| x.0 == entry.path()) {
            return;
        }

        if entry.path().extension().is_none() {
            return;
        }

        let extension = entry
            .path()
            .extension()
            .unwrap()
            .to_string_lossy()
            .to_string();

        if extension != file_extension {
            return;
        }

        r_programs.0.push((
            entry.path().clone(),
            entry
                .path()
                .clone()
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string(),
        ));
    });
}

pub fn load_program(
    mut r_program: ResMut<ActiveProgram>,
    r_chunk_size: Res<ChunkSize>,
) {
    if r_program.file.original.is_none() {
        return;
    }
    r_program
        .file
        .scan_chunk(r_chunk_size.0)
        .iter()
        .for_each(|(symbol, i)| {
            let _ = r_program.symbols.insert(symbol.into(), *i);
        });
}
