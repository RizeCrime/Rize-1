#[allow(unused_imports)]
use bevy::prelude::*;

use crate::{
    constants::{
        FLAG_CARRY, FLAG_NEGATIVE, FLAG_OVERFLOW, FLAG_ZERO,
        N_GENERAL_PURPOSE_REGISTERS, PROGRAM_COUNTER,
    },
    types::{ActiveProgram, Register, Registers},
};

use super::super::Interpreter;

#[derive(Debug, Default)]
pub struct AzmInterpreter {}

#[allow(unused_variables)]
impl Interpreter for AzmInterpreter {
    fn setup_registers(&self, registers: &mut Registers) {
        info!("Setting up Basic Registers...");

        let program_counter = Register::normal(PROGRAM_COUNTER);
        let memory_address_register = Register::normal("mar");
        let memory_data_register = Register::normal("mdr");

        registers.insert(program_counter);
        registers.insert(memory_address_register);
        registers.insert(memory_data_register);

        info!("Finished setting up Basic Registers.");
        info!("Setting up Flags...");

        let zero_flag = Register::flag(FLAG_ZERO);
        let carry_flag = Register::flag(FLAG_CARRY);
        let overflow_flag = Register::flag(FLAG_OVERFLOW);
        let negative_flag = Register::flag(FLAG_NEGATIVE);

        registers.insert(zero_flag);
        registers.insert(carry_flag);
        registers.insert(overflow_flag);
        registers.insert(negative_flag);

        info!("Finished setting up Flags.");
        info!("Setting up General Purpose Registers...");

        for i in 0..N_GENERAL_PURPOSE_REGISTERS {
            // Convert index to letter (0->a, 1->b, etc)
            let letter = (b'a' + i as u8) as char;
            let reg_name = format!("g{}", letter);
            let gpr = Register::normal(reg_name);
            registers.insert(gpr);
        }

        info!("Finished setting up General Purpose Registers.");
    }

    fn load_program(&self, program: &mut ActiveProgram) {}

    fn fetch(
        &self,
        registers: &mut Registers,
        program: &mut ActiveProgram,
    ) -> Option<()> {
        let pc: &mut Register = registers.get(PROGRAM_COUNTER).unwrap();

        let mut lines_iter =
            program.contents.lines().skip(pc.byte.as_decimal() as usize);

        loop {
            if let Some(line) = lines_iter.next() {
                let next = line.trim().to_string();

                if next.is_empty()
                    || next.starts_with('#')
                    || next.starts_with('.')
                {
                    pc.inc().ok()?;
                    continue;
                }

                pc.inc().ok()?;
                break;
            } else {
                warn!("Found EOF.");
                return None;
            }
        }

        Some(())
    }
    fn decode(&self) {}
    fn execute(&self) {}
}
