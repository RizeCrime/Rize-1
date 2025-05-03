use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    constants::{FLAG_CARRY, FLAG_ZERO, PROGRAM_COUNTER},
    display::DisplayMemory,
    types::{
        ActiveProgram, ArgType, ByteOpResult, ByteOperations, ProgramArg,
        Registers, RizeError, RizeErrorType, SystemMemory, DSB,
    },
};

pub const EXPECT_PREVIOUSLY_VERIFIED: &str =
    "Needs to be Verified in the Decode Stage.";

pub fn mov(
    arg1: &ProgramArg, // Destination (Register or MemAddr)
    arg2: &ProgramArg, // Source (Register, Immediate, MemAddr)
    registers: &mut Registers,
    memory: &SystemMemory,
) -> Result<(), RizeError> {
    let source_value: DSB =
        arg2.value.clone().expect(EXPECT_PREVIOUSLY_VERIFIED);

    match &arg1.arg_type {
        ArgType::Register(dest_reg_name) => {
            if let Some(register) = registers.get(&dest_reg_name) {
                register.write(source_value)
            } else {
                return Err(RizeError {
                    type_: RizeErrorType::RegisterRead(format!(
                        "Cannot get a register named \"{}\"",
                        dest_reg_name
                    )),
                });
            }
        }
        ArgType::MemAddr(dest_addr) => {
            if let Some(byte) = memory.bytes.get(dest_addr) {
                byte.write(source_value)?;
                Ok(())
            } else {
                Err(RizeError {
                    type_: RizeErrorType::MemoryRead(format!(
                        "No byte found at address {}",
                        dest_addr
                    )),
                })
            }
        }
        _ => Err(RizeError {
            type_: RizeErrorType::Execute(
                "MOV destination (arg1) must be Register or MemAddr."
                    .to_string(),
            ),
        }),
    }
}

pub fn add(
    arg1: &ProgramArg,
    arg2: &ProgramArg,
    arg3: &ProgramArg,
    registers: &mut Registers,
) -> Result<(), RizeError> {
    let v2 = arg2.value.clone().expect(EXPECT_PREVIOUSLY_VERIFIED);

    if !matches!(arg1.arg_type, ArgType::Register(_)) {
        return Err(RizeError {
            type_: RizeErrorType::Execute(
                "ADD requires the first argument (arg1) to be a register."
                    .to_string(),
            ),
        });
    }

    let target = if matches!(arg3.arg_type, ArgType::Register(_)) {
        registers
            .get(&arg3.arg_type.as_string())
            .expect("Fix any errors that lead here.")
    } else {
        registers
            .get(&arg1.arg_type.as_string())
            .expect("Fix any errors that lead here.")
    };

    let result: ByteOpResult = target.byte.add(v2)?;

    set_flags(registers, result);

    Ok(())
}

pub fn sub(
    arg1: &ProgramArg,
    arg2: &ProgramArg,
    arg3: &ProgramArg,
    registers: &mut Registers,
) -> Result<(), RizeError> {
    let v2 = arg2.value.clone().expect(EXPECT_PREVIOUSLY_VERIFIED);

    if !matches!(arg1.arg_type, ArgType::Register(_)) {
        return Err(RizeError {
            type_: RizeErrorType::Execute(
                "SUB requires the first argument (arg1) to be a register."
                    .to_string(),
            ),
        });
    }

    let target = if matches!(arg3.arg_type, ArgType::Register(_)) {
        registers
            .get(&arg3.arg_type.as_string())
            .expect("Fix any errors that lead here.")
    } else {
        registers
            .get(&arg1.arg_type.as_string())
            .expect("Fix any errors that lead here.")
    };

    let result: ByteOpResult = target.byte.sub(v2)?;

    set_flags(registers, result);

    Ok(())
}

// ---------------- //
// Helper Functions //
// ---------------- //

fn set_flags(registers: &mut Registers, result: ByteOpResult) {
    // Zero Flag
    if result.result == 0u8.into() {
        let _ = registers.get(FLAG_ZERO).unwrap().write(DSB::Flag(true));
    } else {
        let _ = registers.get(FLAG_ZERO).unwrap().write(DSB::Flag(false));
    }

    // Carry Flag
    if result.carry {
        let _ = registers.get(FLAG_CARRY).unwrap().write(DSB::Flag(true));
    } else {
        let _ = registers.get(FLAG_CARRY).unwrap().write(DSB::Flag(false));
    }

    // TODO: Implement Negative and Overflow flags based on ByteOpResult
    // Negative Flag (Placeholder)
    // let _ = registers.get(FLAG_NEGATIVE).unwrap().write(DSB::Flag(false));

    // Overflow Flag (Placeholder)
    // let _ = registers.get(FLAG_OVERFLOW).unwrap().write(DSB::Flag(false));
}

/// Determines the value of an operand (Register, Immediate, or Memory Address).
pub fn get_operand_value(
    registers: &mut Registers,
    memory: &SystemMemory,
    arg: &ArgType,
) -> Result<Option<DSB>, RizeError> {
    match &arg {
        ArgType::Register(reg_name) => {
            if let Some(register) = registers.get(&reg_name) {
                return Ok(Some(register.read()?));
            } else {
                return Err(RizeError {
                    type_: RizeErrorType::RegisterRead(format!(
                        "Cannot get a register named \"{}\"",
                        reg_name
                    )),
                });
            }
        }
        ArgType::Immediate(imm) => return Ok(Some((*imm as u128).into())),
        ArgType::MemAddr(addr) => {
            if let Some(byte) = memory.bytes.get(&(*addr as usize)) {
                return Ok(Some(byte.read()?));
            } else {
                return Err(RizeError {
                    type_: RizeErrorType::MemoryRead(format!(
                        "No byte found at address {}",
                        addr
                    )),
                });
            }
        }
        ArgType::Symbol(_sym) => {
            return Ok(None);
        }
        ArgType::None => {
            return Ok(None);
        }
        _ => {
            return Err(RizeError {
                type_: RizeErrorType::Decode(
                    "Invalid/None ArgType encountered where value operand expected.".to_string(),
                )
            });
        }
    }
}

pub fn mul(
    arg1: &ProgramArg,
    arg2: &ProgramArg,
    arg3: &ProgramArg,
    registers: &mut Registers,
) -> Result<(), RizeError> {
    let v2 = arg2.value.clone().expect(EXPECT_PREVIOUSLY_VERIFIED);

    // Validate arg1 is a register if arg3 is not provided
    if !matches!(arg3.arg_type, ArgType::Register(_))
        && !matches!(arg1.arg_type, ArgType::Register(_))
    {
        return Err(RizeError {
            type_: RizeErrorType::Execute(
                "MUL requires the first argument (arg1) to be a register when the third is omitted.".to_string(),
            ),
        });
    }

    // Determine target register (arg3 or arg1)
    let target = if matches!(arg3.arg_type, ArgType::Register(_)) {
        registers
            .get(&arg3.arg_type.as_string())
            .expect("MUL arg3 register should exist if type is Register")
    } else {
        registers
            .get(&arg1.arg_type.as_string())
            .expect("MUL arg1 register should exist if type is Register")
    };

    let result: ByteOpResult = target.byte.mul(v2)?;

    set_flags(registers, result);

    Ok(())
}

pub fn div(
    arg1: &ProgramArg,
    arg2: &ProgramArg,
    arg3: &ProgramArg,
    registers: &mut Registers,
) -> Result<(), RizeError> {
    let v2 = arg2.value.clone().expect(EXPECT_PREVIOUSLY_VERIFIED);

    // Check for division by zero
    if v2.as_usize() == 0 {
        return Err(RizeError {
            type_: RizeErrorType::Execute("Division by zero".to_string()),
        });
    }

    // Validate arg1 is a register if arg3 is not provided
    if !matches!(arg3.arg_type, ArgType::Register(_))
        && !matches!(arg1.arg_type, ArgType::Register(_))
    {
        return Err(RizeError {
            type_: RizeErrorType::Execute(
                "DIV requires the first argument (arg1) to be a register when the third is omitted.".to_string(),
            ),
        });
    }

    // Determine target register (arg3 or arg1)
    let target = if matches!(arg3.arg_type, ArgType::Register(_)) {
        registers
            .get(&arg3.arg_type.as_string())
            .expect("DIV arg3 register should exist if type is Register")
    } else {
        registers
            .get(&arg1.arg_type.as_string())
            .expect("DIV arg1 register should exist if type is Register")
    };

    let result: ByteOpResult = target.byte.div(v2)?;

    set_flags(registers, result);

    Ok(())
}

pub fn st(
    arg1: &ProgramArg, // Register containing Destination MemAddr
    arg2: &ProgramArg, // Register containing Source Value
    memory: &SystemMemory,
) -> Result<(), RizeError> {
    // Validate arguments are registers
    if !matches!(arg1.arg_type, ArgType::Register(_)) {
        return Err(RizeError {
            type_: RizeErrorType::Execute(
                "ST requires the first argument (arg1) to be a Register (containing the address).".to_string(),
            ),
        });
    }
    if !matches!(arg2.arg_type, ArgType::Register(_)) {
        return Err(RizeError {
            type_: RizeErrorType::Execute(
                "ST requires the second argument (arg2) to be a Register (containing the value).".to_string(),
            ),
        });
    }

    let dest_addr_dsb = arg1.value.clone().expect(EXPECT_PREVIOUSLY_VERIFIED);
    let source_value: DSB =
        arg2.value.clone().expect(EXPECT_PREVIOUSLY_VERIFIED);

    // Convert address DSB to usize
    let dest_addr = dest_addr_dsb.as_usize(); // Potential truncation handled by as_usize

    if let Some(byte) = memory.bytes.get(&dest_addr) {
        byte.write(source_value)?;
        Ok(())
    } else {
        Err(RizeError {
            type_: RizeErrorType::MemoryWrite(format!(
                "ST Error: No byte found at address {} (from reg '{}') to store value (from reg '{}')",
                dest_addr, arg1.arg_type.as_string(), arg2.arg_type.as_string()
            )),
        })
    }
}

pub fn ld(
    arg1: &ProgramArg, // Destination Register
    arg2: &ProgramArg, // Register containing Source MemAddr
    registers: &mut Registers,
    memory: &SystemMemory,
) -> Result<(), RizeError> {
    // Validate arguments are registers
    if !matches!(arg1.arg_type, ArgType::Register(_)) {
        return Err(RizeError {
            type_: RizeErrorType::Execute(
                "LD requires the first argument (arg1) to be the Destination Register.".to_string(),
            ),
        });
    }
    if !matches!(arg2.arg_type, ArgType::Register(_)) {
        return Err(RizeError {
            type_: RizeErrorType::Execute(
                "LD requires the second argument (arg2) to be a Register (containing the address).".to_string(),
            ),
        });
    }

    let dest_reg_name = arg1.arg_type.as_string();
    let source_addr_dsb = arg2.value.clone().expect(EXPECT_PREVIOUSLY_VERIFIED);

    // Convert address DSB to usize
    let source_addr = source_addr_dsb.as_usize(); // Potential truncation handled by as_usize

    // Read value from memory
    let source_value = if let Some(byte) = memory.bytes.get(&source_addr) {
        byte.read()?
    } else {
        return Err(RizeError {
            type_: RizeErrorType::MemoryRead(format!(
                "LD Error: No byte found at address {} (from reg '{}') to load value into reg '{}'",
                source_addr, arg2.arg_type.as_string(), dest_reg_name
            )),
        });
    };

    // Write value to destination register
    if let Some(register) = registers.get(&dest_reg_name) {
        register.write(source_value)?;
        Ok(())
    } else {
        // This error should ideally be caught during Decode if the register name is invalid
        Err(RizeError {
            type_: RizeErrorType::RegisterWrite(format!(
                "LD Error: Cannot get destination register '{}' (Was Decode successful?)",
                dest_reg_name
            )),
        })
    }
}

pub fn and(
    arg1: &ProgramArg,
    arg2: &ProgramArg,
    arg3: &ProgramArg,
    registers: &mut Registers,
) -> Result<(), RizeError> {
    let v2 = arg2.value.clone().expect(EXPECT_PREVIOUSLY_VERIFIED);

    // Validate arg1 is a register if arg3 is not provided
    if !matches!(arg3.arg_type, ArgType::Register(_))
        && !matches!(arg1.arg_type, ArgType::Register(_))
    {
        return Err(RizeError {
            type_: RizeErrorType::Execute(
                "AND requires the first argument (arg1) to be a register when the third is omitted.".to_string(),
            ),
        });
    }

    // Determine target register (arg3 or arg1)
    let target = if matches!(arg3.arg_type, ArgType::Register(_)) {
        registers
            .get(&arg3.arg_type.as_string())
            .expect("AND arg3 register should exist if type is Register")
    } else {
        registers
            .get(&arg1.arg_type.as_string())
            .expect("AND arg1 register should exist if type is Register")
    };

    let result: ByteOpResult = target.byte.bitand(v2)?;

    set_flags(registers, result);

    Ok(())
}

pub fn or(
    arg1: &ProgramArg,
    arg2: &ProgramArg,
    arg3: &ProgramArg,
    registers: &mut Registers,
) -> Result<(), RizeError> {
    let v2 = arg2.value.clone().expect(EXPECT_PREVIOUSLY_VERIFIED);

    // Validate arg1 is a register if arg3 is not provided
    if !matches!(arg3.arg_type, ArgType::Register(_))
        && !matches!(arg1.arg_type, ArgType::Register(_))
    {
        return Err(RizeError {
            type_: RizeErrorType::Execute(
                "OR requires the first argument (arg1) to be a register when the third is omitted.".to_string(),
            ),
        });
    }

    // Determine target register (arg3 or arg1)
    let target = if matches!(arg3.arg_type, ArgType::Register(_)) {
        registers
            .get(&arg3.arg_type.as_string())
            .expect("OR arg3 register should exist if type is Register")
    } else {
        registers
            .get(&arg1.arg_type.as_string())
            .expect("OR arg1 register should exist if type is Register")
    };

    let result: ByteOpResult = target.byte.bitor(v2)?;

    set_flags(registers, result);

    Ok(())
}

pub fn xor(
    arg1: &ProgramArg,
    arg2: &ProgramArg,
    arg3: &ProgramArg,
    registers: &mut Registers,
) -> Result<(), RizeError> {
    let v2 = arg2.value.clone().expect(EXPECT_PREVIOUSLY_VERIFIED);

    // Validate arg1 is a register if arg3 is not provided
    if !matches!(arg3.arg_type, ArgType::Register(_))
        && !matches!(arg1.arg_type, ArgType::Register(_))
    {
        return Err(RizeError {
            type_: RizeErrorType::Execute(
                "XOR requires the first argument (arg1) to be a register when the third is omitted.".to_string(),
            ),
        });
    }

    // Determine target register (arg3 or arg1)
    let target = if matches!(arg3.arg_type, ArgType::Register(_)) {
        registers
            .get(&arg3.arg_type.as_string())
            .expect("XOR arg3 register should exist if type is Register")
    } else {
        registers
            .get(&arg1.arg_type.as_string())
            .expect("XOR arg1 register should exist if type is Register")
    };

    let result: ByteOpResult = target.byte.bitxor(v2)?;

    set_flags(registers, result);

    Ok(())
}

pub fn not(
    arg1: &ProgramArg,
    arg2: &ProgramArg, // Optional destination register
    registers: &mut Registers,
) -> Result<(), RizeError> {
    // Validate arg1 is a register
    if !matches!(arg1.arg_type, ArgType::Register(_)) {
        return Err(RizeError {
            type_: RizeErrorType::Execute(
                "NOT requires the first argument (arg1) to be a register."
                    .to_string(),
            ),
        });
    }

    // Determine target register (arg2 if specified and valid, otherwise arg1)
    let target = if matches!(arg2.arg_type, ArgType::Register(_)) {
        registers
            .get(&arg2.arg_type.as_string())
            .expect("NOT arg2 register should exist if type is Register")
    } else {
        registers
            .get(&arg1.arg_type.as_string())
            .expect("NOT arg1 register should exist")
    };

    let result: ByteOpResult = target.byte.bitnot()?;

    set_flags(registers, result);

    Ok(())
}

pub fn shl(
    arg1: &ProgramArg,
    arg2: &ProgramArg, // Shift amount (Immediate or Register)
    arg3: &ProgramArg, // Optional destination register
    registers: &mut Registers,
) -> Result<(), RizeError> {
    // Validate arg1 is a register if arg3 is not provided
    if !matches!(arg3.arg_type, ArgType::Register(_))
        && !matches!(arg1.arg_type, ArgType::Register(_))
    {
        return Err(RizeError {
            type_: RizeErrorType::Execute(
                "SHL requires the first argument (arg1) to be a register when the third is omitted.".to_string(),
            ),
        });
    }

    // Determine target register (arg3 or arg1)
    let target = if matches!(arg3.arg_type, ArgType::Register(_)) {
        registers
            .get(&arg3.arg_type.as_string())
            .expect("SHL arg3 register should exist if type is Register")
    } else {
        registers
            .get(&arg1.arg_type.as_string())
            .expect("SHL arg1 register should exist if type is Register")
    };

    let shift_amount = arg2.value.clone().expect(EXPECT_PREVIOUSLY_VERIFIED);

    let result: ByteOpResult = target.byte.bitshl(shift_amount)?;

    set_flags(registers, result);

    Ok(())
}

pub fn shr(
    arg1: &ProgramArg,
    arg2: &ProgramArg, // Shift amount (Immediate or Register)
    arg3: &ProgramArg, // Optional destination register
    registers: &mut Registers,
) -> Result<(), RizeError> {
    // Validate arg1 is a register if arg3 is not provided
    if !matches!(arg3.arg_type, ArgType::Register(_))
        && !matches!(arg1.arg_type, ArgType::Register(_))
    {
        return Err(RizeError {
            type_: RizeErrorType::Execute(
                "SHR requires the first argument (arg1) to be a register when the third is omitted.".to_string(),
            ),
        });
    }

    // Determine target register (arg3 or arg1)
    let target = if matches!(arg3.arg_type, ArgType::Register(_)) {
        registers
            .get(&arg3.arg_type.as_string())
            .expect("SHR arg3 register should exist if type is Register")
    } else {
        registers
            .get(&arg1.arg_type.as_string())
            .expect("SHR arg1 register should exist if type is Register")
    };

    let shift_amount = arg2.value.clone().expect(EXPECT_PREVIOUSLY_VERIFIED);

    let result: ByteOpResult = target.byte.bitshr(shift_amount)?;

    set_flags(registers, result);

    Ok(())
}

pub fn wdm(
    arg1: &ProgramArg, // RG (Register or Immediate, U16 with R in high byte, G in low)
    arg2: &ProgramArg, // BA (Register or Immediate, U16 with B in high byte, A in low)
    arg3: &ProgramArg, // XY (Register or Immediate, U16 with X in high byte, Y in low)
    display_memory: &mut DisplayMemory,
) -> Result<(), RizeError> {
    let rg_val = arg1
        .value
        .clone()
        .expect(EXPECT_PREVIOUSLY_VERIFIED)
        .as_usize() as u16;
    let ba_val = arg2
        .value
        .clone()
        .expect(EXPECT_PREVIOUSLY_VERIFIED)
        .as_usize() as u16;
    let xy_val = arg3
        .value
        .clone()
        .expect(EXPECT_PREVIOUSLY_VERIFIED)
        .as_usize() as u16;

    // Extract individual 8-bit values assuming U16 packing
    // Masking with 0xFF ensures we only take the lower 8 bits.
    // Right-shifting by 8 gets the upper 8 bits.
    let r = (rg_val >> 8) as u8;
    let g = (rg_val & 0xFF) as u8;
    let b = (ba_val >> 8) as u8;
    let a = (ba_val & 0xFF) as u8;
    let x = (xy_val >> 8) as u8;
    let y = (xy_val & 0xFF) as u8;

    display_memory.set_pixel(x, y, [r, g, b, a])

    // WDM does not typically affect flags
    // No call to set_flags()
}

pub fn jmp(
    arg1: &ProgramArg,
    registers: &mut Registers,
    symbols: &HashMap<String, usize>,
) -> Result<(), RizeError> {
    if !matches!(arg1.arg_type, ArgType::Symbol(_)) {
        return Err(RizeError {
            type_: RizeErrorType::Execute(
                "JMP requires the first argument (arg1) to be a symbol.".into(),
            ),
        });
    }

    let symbol: String = arg1.arg_type.as_string();
    let target: usize = *symbols.get(&symbol).unwrap();

    let pc = registers.get(PROGRAM_COUNTER).unwrap();
    pc.write(target)
}

pub fn jiz(
    arg1: &ProgramArg,
    registers: &mut Registers,
    symbols: &HashMap<String, usize>,
) -> Result<(), RizeError> {
    // Validate arg1 is a symbol
    let symbol = match &arg1.arg_type {
        ArgType::Symbol(s) => s.clone(),
        _ => {
            return Err(RizeError {
                type_: RizeErrorType::Execute(
                    "JIZ requires the first argument (arg1) to be a symbol."
                        .into(),
                ),
            });
        }
    };

    // Check the Zero Flag
    let zero_flag = registers
        .get(FLAG_ZERO)
        .ok_or_else(|| RizeError {
            type_: RizeErrorType::RegisterRead(format!(
                "Could not find Zero Flag ('{}')",
                FLAG_ZERO
            )),
        })?
        .read()?;

    if zero_flag.as_usize() == 1 {
        // If zero flag is set, jump
        let target = *symbols.get(&symbol).ok_or_else(|| RizeError {
            type_: RizeErrorType::Execute(format!(
                "JIZ Error: Symbol '{}' not found.",
                symbol
            )),
        })?;

        let pc = registers.get(PROGRAM_COUNTER).ok_or_else(|| RizeError {
            type_: RizeErrorType::RegisterRead(format!(
                "Could not find Program Counter ('{}')",
                PROGRAM_COUNTER
            )),
        })?;
        pc.write(target)?;
    }
    // If zero flag is not set, do nothing (continue to next instruction)
    Ok(())
}

pub fn jin(
    arg1: &ProgramArg,
    registers: &mut Registers,
    symbols: &HashMap<String, usize>,
) -> Result<(), RizeError> {
    // Validate arg1 is a symbol
    let symbol = match &arg1.arg_type {
        ArgType::Symbol(s) => s.clone(),
        _ => {
            return Err(RizeError {
                type_: RizeErrorType::Execute(
                    "JIN requires the first argument (arg1) to be a symbol."
                        .into(),
                ),
            });
        }
    };

    // Check the Negative Flag
    let negative_flag = registers
        .get(crate::constants::FLAG_NEGATIVE) // Use crate path for clarity
        .ok_or_else(|| RizeError {
            type_: RizeErrorType::RegisterRead(format!(
                "Could not find Negative Flag ('{}')",
                crate::constants::FLAG_NEGATIVE
            )),
        })?
        .read()?;

    if negative_flag.as_usize() == 1 {
        // If negative flag is set, jump
        let target = *symbols.get(&symbol).ok_or_else(|| RizeError {
            type_: RizeErrorType::Execute(format!(
                "JIN Error: Symbol '{}' not found.",
                symbol
            )),
        })?;

        let pc = registers.get(PROGRAM_COUNTER).ok_or_else(|| RizeError {
            type_: RizeErrorType::RegisterRead(format!(
                "Could not find Program Counter ('{}')",
                PROGRAM_COUNTER
            )),
        })?;
        pc.write(target)?;
    }
    // If negative flag is not set, do nothing
    Ok(())
}
