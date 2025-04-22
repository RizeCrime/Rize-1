use bevy::prelude::*;

use crate::*;

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}

pub fn setup_registers(mut r_registers: ResMut<Registers>) {
    info!("Setting up Basic Registers...");

    let instruction_register = Register::init(INSTRUCTION_WIDTH);
    let program_counter = Register::init(CPU_BITTAGE);
    let memory_address_register = Register::init(CPU_BITTAGE);
    let memory_data_register = Register::init(CPU_BITTAGE);

    r_registers
        .as_mut()
        .insert(PROGRAM_COUNTER.into(), program_counter);
    r_registers
        .as_mut()
        .insert("mar".into(), memory_address_register);
    r_registers
        .as_mut()
        .insert("mdr".into(), memory_data_register);

    info!("Finished setting up Basic Registers.");

    info!("Setting up Flags...");

    let zero_flag = Register::init(1);
    let carry_flag = Register::init(1);
    let overflow_flag = Register::init(1);
    let negative_flag = Register::init(1);

    r_registers.as_mut().insert(FLAG_ZERO.into(), zero_flag);
    r_registers.as_mut().insert(FLAG_CARRY.into(), carry_flag);
    r_registers
        .as_mut()
        .insert(FLAG_OVERFLOW.into(), overflow_flag);
    r_registers
        .as_mut()
        .insert(FLAG_NEGATIVE.into(), negative_flag);

    info!("Finished setting up Flags.");
    info!("Setting up General Purpose Registers...");

    for i in 0..N_GENERAL_PURPOSE_REGISTERS {
        // Convert index to letter (0->a, 1->b, etc)
        let letter = (b'a' + i as u8) as char;
        let reg_name = format!("g{}", letter);
        let gpr = Register::init(CPU_BITTAGE);
        r_registers.as_mut().insert(reg_name, gpr);
    }

    info!("Finished setting up General Purpose Registers.");
}
