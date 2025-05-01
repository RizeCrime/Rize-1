use std::{os::raw, str::FromStr};

#[allow(unused_imports)]
use bevy::prelude::*;

use crate::DisplayMemory;
use crate::{
    constants::{
        FLAG_CARRY, FLAG_NEGATIVE, FLAG_OVERFLOW, FLAG_ZERO,
        N_GENERAL_PURPOSE_REGISTERS, PROGRAM_COUNTER,
    },
    types::{
        ActiveProgram, ArgType, OpCode, Register, Registers, SystemMemory,
    },
    CpuCycleStage,
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
    fn decode(&self, program: &mut ActiveProgram) {
        let parts: Vec<&str> = program.line.split_whitespace().collect();
        let raw_opcode = parts.get(0).copied().unwrap_or_default().to_string();
        let raw_arg1 = parts.get(1).copied().unwrap_or_default().to_string();
        let raw_arg2 = parts.get(2).copied().unwrap_or_default().to_string();
        let raw_arg3 = parts.get(3).copied().unwrap_or_default().to_string();

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
                        Some((symbol_name.to_string(), n + 1))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        if let Ok(opcode) = OpCode::from_str(raw_opcode.as_str()) {
            program.opcode = opcode;
        }

        program.arg1 = ArgType::from_string(raw_arg1);
        program.arg2 = ArgType::from_string(raw_arg2);
        program.arg3 = ArgType::from_string(raw_arg3);
    }
    fn execute(
        &self,
        program: &mut ActiveProgram,
        registers: &mut Registers,
        memory: &mut SystemMemory,
        display_memory: &mut DisplayMemory,
        images: &Assets<Image>,
        mut next_cpu_stage: ResMut<NextState<CpuCycleStage>>,
    ) -> Option<()> {
        //     let execution_result = match program.opcode {
        //         OpCode::MOV => mov(
        //             &program.arg1,
        //             &program.arg2,
        //             registers,
        //             memory,
        //         ),
        //         OpCode::ADD => {
        //             add(
        //                 &program.arg1,
        //                 &program.arg2,
        //                 &arg3_option,
        //                 registers,
        //             )
        //         }
        //         OpCode::SUB => {
        //             sub(
        //                 &program.arg1,
        //                 &program.arg2,
        //                 &arg3_option,
        //                 registers,
        //                 &r_memory,
        //             )
        //         }
        //         OpCode::MUL => {
        //             mul(
        //                 &program.arg1,
        //                 &program.arg2,
        //                 &arg3_option,
        //                 registers,
        //                 &r_memory,
        //             )
        //         }
        //         OpCode::DIV => {
        //             div(
        //                 &program.arg1,
        //                 &program.arg2,
        //                 &arg3_option,
        //                 registers,
        //                 &r_memory,
        //             )
        //         }
        //         OpCode::ST => st(registers, memory),
        //         OpCode::LD => ld(registers, memory),
        //         OpCode::AND => {
        //             and(
        //                 &program.arg1,
        //                 &program.arg2,
        //                 &arg3_option,
        //                 registers,
        //             )
        //         }
        //         OpCode::OR => {
        //             let arg3_option = if program.arg3.is_empty() {
        //                 None
        //             } else {
        //                 Some(program.arg3.clone())
        //             };
        //             or(
        //                 &program.arg1,
        //                 &program.arg2,
        //                 &arg3_option,
        //                 registers,
        //             )
        //         }
        //         OpCode::XOR => {
        //             let arg3_option = if program.arg3.is_empty() {
        //                 None
        //             } else {
        //                 Some(program.arg3.clone())
        //             };
        //             xor(
        //                 &program.arg1,
        //                 &program.arg2,
        //                 &arg3_option,
        //                 registers,
        //             )
        //         }
        //         OpCode::NOT => not(&program.arg1, registers),
        //         OpCode::SHL => {
        //             shl(&program.arg1, &program.arg2, registers)
        //         }
        //         OpCode::SHR => {
        //             shr(&program.arg1, &program.arg2, registers)
        //         }
        //         OpCode::HALT => {
        //             info!("Halting CPU!");
        //             next_cpu_stage.set(CpuCycleStage::Halt);
        //             Ok(())
        //         }
        //         OpCode::WDM => wdm(
        //             &program.arg1,
        //             &program.arg2,
        //             &program.arg3,
        //             r_display_memory,
        //             registers,
        //             memory,
        //         ),
        //         OpCode::JMP => {
        //             let target_symbol =
        //                 program.arg1.strip_prefix('.').unwrap_or_default();
        //             let target_line = program
        //                 .symbols
        //                 .get(target_symbol)
        //                 .copied()
        //                 .unwrap_or_default();
        //             if target_line == 0 {
        //                 Err(RizeError {
        //                     type_: RizeErrorType::Execute,
        //                     message: format!(
        //                         "JMP target symbol '.{}' not found.",
        //                         target_symbol
        //                     ),
        //                 })
        //             } else {
        //                 program.line = target_line;
        //                 match get_register_mut(registers, PROGRAM_COUNTER) {
        //                     Ok(pc_reg) => pc_reg.write_section_u16(target_line as u16),
        //                     Err(e) => Err(e),
        //                 }
        //             }
        //         }
        //         OpCode::JIZ => {
        //             // Read flag, handling Result
        //             match get_operand_value(
        //                 registers,
        //                 &Memory::new(),
        //                 &ArgType::Register(FLAG_ZERO.to_string()),
        //             ) {
        //                 Ok(zero_flag) => {
        //                     if zero_flag == 1 {
        //                         // Jump logic
        //                         let target_symbol = program
        //                             .arg1
        //                             .raw
        //                             .strip_prefix('.')
        //                             .unwrap_or_default();
        //                         let target_line = program
        //                             .symbols
        //                             .get(target_symbol)
        //                             .copied()
        //                             .unwrap_or_default();
        //                         if target_line == 0 {
        //                             Err(RizeError {
        //                                 type_: RizeErrorType::Execute,
        //                                 message: format!(
        //                                     "JIZ target symbol '.{}' not found.",
        //                                     target_symbol
        //                                 ),
        //                             })
        //                         } else {
        //                             program.line = target_line;
        //                             match get_register_mut(registers, PROGRAM_COUNTER) {
        //                                 Ok(pc_reg) => {
        //                                     pc_reg.write_section_u16(target_line as u16)
        //                                 }
        //                                 Err(e) => Err(e),
        //                             }
        //                         }
        //                     } else {
        //                         // Flag is zero, don't jump
        //                         Ok(())
        //                     }
        //                 }
        //                 Err(e) => Err(e), // Propagate flag read error
        //             }
        //         }
        //         OpCode::JIN => {
        //             // Read flag, handling Result
        //             match get_operand_value(
        //                 registers,
        //                 &Memory::new(),
        //                 &ArgType::Register(FLAG_NEGATIVE.to_string()),
        //             ) {
        //                 Ok(negative_flag) => {
        //                     if negative_flag == 1 {
        //                         // Jump logic
        //                         let target_symbol = program
        //                             .arg1
        //                             .strip_prefix('.')
        //                             .unwrap_or_default();
        //                         let target_line = program
        //                             .symbols
        //                             .get(target_symbol)
        //                             .copied()
        //                             .unwrap_or_default();
        //                         if target_line == 0 {
        //                             Err(RizeError {
        //                                 type_: RizeErrorType::Execute,
        //                                 message: format!(
        //                                     "JIN target symbol '.{}' not found.",
        //                                     target_symbol
        //                                 ),
        //                             })
        //                         } else {
        //                             program.line = target_line;
        //                             match get_register_mut(registers, PROGRAM_COUNTER) {
        //                                 Ok(pc_reg) => {
        //                                     pc_reg.write_section_u16(target_line as u16)
        //                                 }
        //                                 Err(e) => Err(e),
        //                             }
        //                         }
        //                     } else {
        //                         // Flag is zero, don't jump
        //                         Ok(())
        //                     }
        //                 }
        //                 Err(e) => Err(e), // Propagate flag read error
        //             }
        //         }
        //         _ => {
        //             warn!("OpCode {:?} not yet implemented!", program.opcode);
        //             Err(RizeError {
        //                 type_: RizeErrorType::Execute,
        //                 message: format!("OpCode {:?} not implemented", program.opcode),
        //             })
        //         }
        //     };

        //     // Handle the result of the execution
        //     if let Err(e) = execution_result {
        //         error!(
        //             "Execution Error ({:?}): {} (Op: {:?}, Args: '{:?}', '{:?}', '{:?}')",
        //             e.type_,
        //             e.message,
        //             program.opcode,
        //             program.arg1,
        //             program.arg2,
        //             program.arg3
        //         );
        //         return None
        //     }
        Some(())
    }
}
