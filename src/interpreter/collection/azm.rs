#[allow(unused_imports)]
use bevy::prelude::*;

use std::str::FromStr;

use crate::{
    constants::{
        FLAG_CARRY, FLAG_NEGATIVE, FLAG_OVERFLOW, FLAG_ZERO,
        N_GENERAL_PURPOSE_REGISTERS, PROGRAM_COUNTER,
    },
    types::{
        ActiveProgram, ArgType, OpCode, Register, Registers, RizeError, RizeErrorType, SystemMemory,
    }, DisplayMemory,
};

use super::{opcode_fn::{add, get_operand_value, mov}, super::Interpreter};

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

                // Save line for decoding cycle
                program.line = next;
                pc.inc().ok()?;
                break;
            } else {
                warn!("Found EOF.");
                return None;
            }
        }

        Some(())
    }
    fn decode(
        &self,
        program: &mut ActiveProgram,
        registers: &mut Registers,
        memory: &mut SystemMemory,
    ) -> Result<(), RizeError> {
        // Retrieving different instruction parts and assigning it to different variables
        let parts: Vec<&str> = program.line.split_whitespace().collect();
        let raw_opcode = if let Some(string_opcode) = parts.get(0).copied() {
            string_opcode.to_string()
        } else {
            return Err(RizeError {
                type_: RizeErrorType::Decode(format!("Couldn't retrieve an opcode from instruction: \"{}\".", program.line))
            });
        };
        let raw_arg1 = if let Some(arg) = parts.get(1).copied() {
            arg.to_string()
        } else {
            return Err(RizeError {
                type_: RizeErrorType::Decode(format!("Couldn't retrieve the first argument from instruction: \"{}\".", program.line))
            });
        };
        let raw_arg2 = if let Some(arg) = parts.get(2).copied() {
            arg.to_string()
        } else {
            return Err(RizeError {
                type_: RizeErrorType::Decode(format!("Couldn't retrieve the second argument from instruction: \"{}\".", program.line))
            });
        };
        let raw_arg3 = if let Some(arg) = parts.get(3).copied() {
            arg.to_string()
        } else {
            String::new()
        };

        // Retrieving symbols (e.g. labels) and saving them
        program.symbols = program
            .contents
            .lines()
            .enumerate()
            .filter_map(|(n, line)| {
                let trimmed_line = line.trim();
                if trimmed_line.starts_with('.') {
                    let symbol_name = &trimmed_line[1..];
                    if symbol_name.len() > 0
                        && symbol_name.len() <= 16
                        && symbol_name.chars().all(char::is_alphabetic)
                    {
                        // Save the symbol name with the line in which it is located
                        Some((symbol_name.to_string(), n + 1))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        // Store processed OpCode into ActiveProgram, to retrieve it at Execute stage
        if let Ok(opcode) = OpCode::from_str(raw_opcode.as_str()) {
            program.opcode = opcode;
        }

        // Validate arguments and get their values (or get None if not available)
        program.arg1.arg_type = ArgType::from_string(raw_arg1);
        program.arg2.arg_type = ArgType::from_string(raw_arg2);
        program.arg3.arg_type = ArgType::from_string(raw_arg3);
        program.arg1.value = get_operand_value(
            registers,
            memory,
            &program.arg1.arg_type,
        )?;
        program.arg2.value = get_operand_value(
            registers,
            memory,
            &program.arg2.arg_type,
        )?;
        program.arg3.value = get_operand_value(
            registers,
            memory,
            &program.arg3.arg_type,
        )?;

        Ok(())
    }
    fn execute(
        &self,
        program: &mut ActiveProgram,
        registers: &mut Registers,
        memory: &mut SystemMemory,
        display_memory: &mut DisplayMemory,
        images: &Assets<Image>,
    ) -> Option<()> {
        let execution_result = match program.opcode {
            OpCode::MOV => mov(&program.arg1, &program.arg2, registers, memory),
            OpCode::ADD => add(&program.arg1, &program.arg2, &program.arg3, registers),
            _ => {
                warn!("OpCode {:?} not yet implemented!", program.opcode);
                Err(RizeError {
                    type_: RizeErrorType::Execute(format!(
                        "OpCode {:?} not implemented",
                        program.opcode
                    )),
                })
            }
        };

        // Handle the result of the execution
        if let Err(e) = execution_result {
            error!(
                "Execution Error ({:?}), (Op: {:?}, Args: '{:?}', '{:?}', '{:?}')",
                e.type_, program.opcode, program.arg1, program.arg2, program.arg3
            );
            return None;
        }
        Some(())
    }
}