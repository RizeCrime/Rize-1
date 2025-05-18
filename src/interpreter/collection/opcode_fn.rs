use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    constants::{FLAG_CARRY, FLAG_ZERO, PROGRAM_COUNTER},
    display::DisplayMemory,
    types::{
        ArgType, Byte, ByteOpResult, ByteOperations, ProgramArg, Registers,
        RizeError, RizeErrorType, SystemMemory, DSB,
    },
};

pub const EXPECT_PREVIOUSLY_VERIFIED: &str =
    "Needs to be Verified in the Decode Stage.";

pub fn mov(
    arg1: &ProgramArg, // Destination (Register or MemAddr)
    arg2: &ProgramArg, // Source (Register, Immediate, MemAddr)
    registers: &mut Registers,
    memory: &mut SystemMemory,
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
            let byte = memory
                .bytes
                .entry(*dest_addr)
                .or_insert_with(|| Byte::default());
            byte.write(source_value)?;
            Ok(())
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
    registers: &mut Registers,
) -> Result<(), RizeError> {
    if !matches!(arg1.arg_type, ArgType::Register(_)) {
        return Err(RizeError {
            type_: RizeErrorType::Execute(
                "ADD requires arg1 to be a Register.".to_string(),
            ),
        });
    }

    let v2 = match &arg2.arg_type {
        ArgType::Immediate(i) => DSB::from_cpu_bittage(*i),
        ArgType::Register(r) => {
            let dsb: DSB = registers
                .get(&r)
                .expect("ADD arg2 Register must exist.")
                .read();
            dsb
        }
        _ => {
            return Err(RizeError {
                type_: RizeErrorType::Execute(
                    "ADD requires arg2 to be either a Register, or an Immediate."
                        .to_string(),
                ),
            });
        }
    };

    let reg1 = registers
        .get(&arg1.arg_type.as_string())
        .expect("ADD arg1 register must exist");

    let result = reg1.byte.add(v2)?;

    set_flags(registers, result);

    Ok(())
}

pub fn sub(
    arg1: &ProgramArg,
    arg2: &ProgramArg,
    registers: &mut Registers,
) -> Result<(), RizeError> {
    if !matches!(arg1.arg_type, ArgType::Register(_)) {
        return Err(RizeError {
            type_: RizeErrorType::Execute(
                "SUB requires arg1 to be a Register.".to_string(),
            ),
        });
    }

    let v2: DSB = match &arg2.arg_type {
        ArgType::Immediate(i) => DSB::from_cpu_bittage(*i),
        ArgType::Register(r) => {
            let dsb: DSB = registers
                .get(&r)
                .expect("SUB arg2 Register must exist.")
                .read();
            dsb
        }
        _ => {
            return Err(RizeError {
                type_: RizeErrorType::Execute(
                    "SUB requires arg2 to be either a Register, or an Immediate."
                    .to_string()
                )
            });
        }
    };

    let reg1 = registers
        .get(&arg1.arg_type.as_string())
        .expect("SUB arg1 register must exist");

    let result = reg1.byte.sub(v2)?;

    set_flags(registers, result);

    Ok(())
}

pub fn mul(
    arg1: &ProgramArg,
    arg2: &ProgramArg,
    registers: &mut Registers,
) -> Result<(), RizeError> {
    if !matches!(arg1.arg_type, ArgType::Register(_)) {
        return Err(RizeError {
            type_: RizeErrorType::Execute(
                "MUL requires arg1 to be a Register.".to_string(),
            ),
        });
    }

    let v2 = match &arg2.arg_type {
        ArgType::Immediate(i) => DSB::from_cpu_bittage(*i),
        ArgType::Register(r) => {
            let dsb: DSB = registers
                .get(&r)
                .expect("MUL arg2 Register must exist.")
                .read();
            dsb
        }
        _ => {
            return Err(RizeError {
                type_: RizeErrorType::Execute(
                    "MUL requires arg2 to be either a Register, or an Immediate."
                        .to_string()
                )
            });
        }
    };

    let reg1 = registers
        .get(&arg1.arg_type.as_string())
        .expect("MUL arg1 register must exist");

    let result = reg1.byte.mul(v2)?;

    set_flags(registers, result);

    Ok(())
}

pub fn div(
    arg1: &ProgramArg,
    arg2: &ProgramArg,
    registers: &mut Registers,
) -> Result<(), RizeError> {
    if !matches!(arg1.arg_type, ArgType::Register(_)) {
        return Err(RizeError {
            type_: RizeErrorType::Execute(
                "DIV requires the first argument (arg1) to be a register."
                    .to_string(),
            ),
        });
    }

    let v2 = match &arg2.arg_type {
        ArgType::Immediate(i) => DSB::from_cpu_bittage(*i),
        ArgType::Register(r) => {
            let dsb: DSB = registers
                .get(&r)
                .expect("DIV arg2 Register must exist.")
                .read();
            dsb
        }
        _ => {
            return Err(RizeError {
                type_: RizeErrorType::Execute(
                    "DIV requires arg2 to be either a Register, or an Immediate."
                        .to_string()
                )
            });
        }
    };

    let reg1 = registers
        .get(&arg1.arg_type.as_string())
        .expect("DIV arg1 register must exist.");

    let result = reg1.byte.div(v2)?;

    set_flags(registers, result);

    Ok(())
}

pub fn st(
    registers: &mut Registers,
    memory: &mut SystemMemory,
) -> Result<(), RizeError> {
    memory.write(
        registers
            .get("MAR")
            .expect("Memory Address Register must exist.")
            .read()
            .as_usize(),
        registers
            .get("MDR")
            .expect("Memory Data Register must exist.")
            .read(),
    );

    Ok(())
}

pub fn ld(
    registers: &mut Registers,
    memory: &mut SystemMemory,
) -> Result<(), RizeError> {
    registers
        .get("MAR")
        .expect("Memory Address Register must exist.")
        .read()
        .as_usize();

    let addr: usize = registers
        .get("MAR")
        .expect("Memory Address Register must exist.")
        .read()
        .as_usize();

    registers
        .get("MDR")
        .expect("Memory Data Register must exist.")
        .write(memory.read(addr))
}

pub fn and(
    arg1: &ProgramArg,
    arg2: &ProgramArg,
    registers: &mut Registers,
) -> Result<(), RizeError> {
    if !matches!(arg1.arg_type, ArgType::Register(_)) {
        return Err(RizeError {
            type_: RizeErrorType::Execute(
                "AND requires arg1 to be a Register.".to_string(),
            ),
        });
    }

    let v2: DSB = match &arg2.arg_type {
        ArgType::Immediate(i) => DSB::from_cpu_bittage(*i),
        ArgType::Register(r) => {
            let dsb: DSB = registers.get(&r).expect("AND requires arg2 to be either a Register, or an Immediate.").read();
            dsb
        }
        _ => {
            return Err(RizeError { type_: RizeErrorType::Execute("AND requires arg2 to be either a Register, or an Immediate.".to_string()) });
        }
    };

    let reg1 = registers
        .get(&arg1.arg_type.as_string())
        .expect("AND arg1 register must exist");

    // Perform bitwise AND, result is written in-place to reg1.byte
    let result = reg1.byte.bitand(v2)?;

    // --- Set flags ---
    set_flags(registers, result);

    Ok(())
}

pub fn or(
    arg1: &ProgramArg,
    arg2: &ProgramArg,
    registers: &mut Registers,
) -> Result<(), RizeError> {
    let v2 = arg2.value.clone().expect(EXPECT_PREVIOUSLY_VERIFIED);

    // Validate arg1 is a register
    if !matches!(arg1.arg_type, ArgType::Register(_)) {
        return Err(RizeError {
            type_: RizeErrorType::Execute(
                "OR requires the first argument (arg1) to be a register."
                    .to_string(),
            ),
        });
    }

    let reg1 = registers
        .get(&arg1.arg_type.as_string())
        .expect("OR arg1 register must exist");

    // Perform bitwise OR, result is written in-place to reg1.byte
    let result = reg1.byte.bitor(v2)?;

    // --- Set flags ---
    set_flags(registers, result);

    Ok(())
}

pub fn xor(
    arg1: &ProgramArg,
    arg2: &ProgramArg,
    registers: &mut Registers,
) -> Result<(), RizeError> {
    let v2 = arg2.value.clone().expect(EXPECT_PREVIOUSLY_VERIFIED);

    // Validate arg1 is a register
    if !matches!(arg1.arg_type, ArgType::Register(_)) {
        return Err(RizeError {
            type_: RizeErrorType::Execute(
                "XOR requires the first argument (arg1) to be a register."
                    .to_string(),
            ),
        });
    }

    let reg1 = registers
        .get(&arg1.arg_type.as_string())
        .expect("XOR arg1 register must exist");

    // Perform bitwise XOR, result is written in-place to reg1.byte
    let result = reg1.byte.bitxor(v2)?;

    // --- Set flags ---
    set_flags(registers, result);

    Ok(())
}

pub fn not(
    arg1: &ProgramArg,
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

    let reg1 = registers
        .get(&arg1.arg_type.as_string())
        .expect("NOT arg1 register must exist");

    // Perform bitwise NOT, result is written in-place to reg1.byte
    let result = reg1.byte.bitnot()?;

    // --- Set flags ---
    set_flags(registers, result);

    Ok(())
}

pub fn shl(
    arg1: &ProgramArg,
    arg2: &ProgramArg, // Shift amount (Immediate or Register)
    registers: &mut Registers,
) -> Result<(), RizeError> {
    // Validate arg1 is a register
    if !matches!(arg1.arg_type, ArgType::Register(_)) {
        return Err(RizeError {
            type_: RizeErrorType::Execute(
                "SHL requires the first argument (arg1) to be a register."
                    .to_string(),
            ),
        });
    }

    let shift_amount = arg2.value.clone().expect(EXPECT_PREVIOUSLY_VERIFIED);

    let reg1 = registers
        .get(&arg1.arg_type.as_string())
        .expect("SHL arg1 register must exist");

    // Perform bitwise SHL, result is written in-place to reg1.byte
    let result = reg1.byte.bitshl(shift_amount)?;

    // --- Set flags ---
    set_flags(registers, result);

    Ok(())
}

pub fn shr(
    arg1: &ProgramArg,
    arg2: &ProgramArg, // Shift amount (Immediate or Register)
    registers: &mut Registers,
) -> Result<(), RizeError> {
    // Validate arg1 is a register
    if !matches!(arg1.arg_type, ArgType::Register(_)) {
        return Err(RizeError {
            type_: RizeErrorType::Execute(
                "SHR requires the first argument (arg1) to be a register."
                    .to_string(),
            ),
        });
    }

    let shift_amount = arg2.value.clone().expect(EXPECT_PREVIOUSLY_VERIFIED);

    let reg1 = registers
        .get(&arg1.arg_type.as_string())
        .expect("SHR arg1 register must exist");

    // Perform bitwise SHR, result is written in-place to reg1.byte
    let result = reg1.byte.bitshr(shift_amount)?;

    // --- Set flags ---
    set_flags(registers, result);

    Ok(())
}

pub fn wdm(
    arg1: &ProgramArg, // RG (Register or Immediate, U16 with R in high byte, G in low)
    arg2: &ProgramArg, // BA (Register or Immediate, U16 with B in high byte, A in low)
    arg3: &ProgramArg,

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

    // WDM does not typiy affect flags
    // No  to set_flags()
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
        .read();

    debug!(
        "src/interpreter/collection/opcode_fn.rs/jiz - zero_flag: {:?}",
        zero_flag
    );

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
        .read();

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

// ---------------- //
// Helper Functions //
// ---------------- //

fn set_flags(registers: &mut Registers, result: ByteOpResult) {
    // Zero Flag
    if result.result.as_usize() == 0 {
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
                return Ok(Some(register.read()));
            } else {
                return Err(RizeError {
                    type_: RizeErrorType::RegisterRead(format!(
                        "Cannot get a register named \"{}\"",
                        reg_name
                    )),
                });
            }
        }
        ArgType::Immediate(imm) => {
            return Ok(Some(DSB::from_cpu_bittage(*imm)))
        }
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
