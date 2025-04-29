use bevy::prelude::*;

use crate::{
    constants::{
        FLAG_CARRY, FLAG_NEGATIVE, FLAG_OVERFLOW, FLAG_ZERO,
        N_GENERAL_PURPOSE_REGISTERS, PROGRAM_COUNTER,
    },
    types::{Register, Registers},
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
