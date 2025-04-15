use bevy::prelude::*;

use crate::*;

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}


pub fn setup_registers(mut registers: ResMut<Registers>) {

    println!("Setting up Basic Registers...");

    let instruction_register = Register::init(INSTRUCTION_WIDTH);
    let program_counter = Register::init(CPU_BITTAGE);
    let memory_address_register = Register::init(CPU_BITTAGE);
    let memory_data_register = Register::init(CPU_BITTAGE);

    registers.as_mut().insert("ir".into(), instruction_register);
    registers.as_mut().insert("pc".into(), program_counter);
    registers.as_mut().insert("mar".into(), memory_address_register);
    registers.as_mut().insert("mdr".into(), memory_data_register);

    println!("Finished setting up Basic Registers.");

    println!("Setting up Flags...");

    let zero_flag = Register::init(1);
    let carry_flag = Register::init(1);
    let overflow_flag = Register::init(1);
    let negative_flag = Register::init(1);

    registers.as_mut().insert("zf".into(), zero_flag);
    registers.as_mut().insert("cf".into(), carry_flag);
    registers.as_mut().insert("of".into(), overflow_flag);
    registers.as_mut().insert("nf".into(), negative_flag);

    println!("Finished setting up Flags.");

    println!("Setting up General Purpose Registers...");

    for i in 0..N_GENERAL_PURPOSE_REGISTERS {
        // Convert index to letter (0->a, 1->b, etc)
        let letter = (b'a' + i as u8) as char;
        let reg_name = format!("g{}", letter);
        
        let gpr = Register::init(CPU_BITTAGE);
        registers.as_mut().insert(reg_name, gpr);
    }

    println!("Finished setting up General Purpose Registers.");


    

}