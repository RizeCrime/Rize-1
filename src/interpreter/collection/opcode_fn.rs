use std::ops::Add;

use crate::{
    constants::{FLAG_CARRY, FLAG_NEGATIVE, FLAG_OVERFLOW, FLAG_ZERO},
    types::{
        ArgType, ByteOperations, ProgramArg, Register, Registers, RizeError, RizeErrorType,
        SystemMemory, DSB,
    },
};

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
                    ))
                });
            }
        }
        ArgType::Immediate(imm) => return Ok(Some(DSB::U16(*imm))),
        ArgType::MemAddr(addr) => {
            if let Some(byte) = memory.bytes.get(&(*addr as usize)) {
                return Ok(Some(byte.read()?));
            } else {
                return Err(RizeError {
                    type_: RizeErrorType::MemoryRead(format!("No byte found at address {}", addr))
                });
            }
        }
        ArgType::Symbol(sym) => {
            return Err(RizeError {
                type_: RizeErrorType::Decode(format!(
                    "Cannot use symbol '.{}' as an operand value.",
                    sym
                ))
            });
        }
        ArgType::None => {
            return Ok(None);
        }
        ArgType::Error => {
            return Err(RizeError {
                type_: RizeErrorType::Decode(
                    "Invalid/None ArgType encountered where value operand expected.".to_string(),
                )
            });
        }
    }
}

///  Determines the destination register for instructions with an optional 3rd argument.
///  Returns a mutable reference to the destination register with 
///  the name of the destination register as a String.
fn determine_destination_register_mut<'a>(
    registers: &'a mut Registers,
    arg1: &ArgType,
    arg3: &ArgType,
) -> Result<(&'a mut Register, String), RizeError> {
    match arg3 {
        ArgType::Register(reg3_name) => {
            if let Some(reg_ref) = registers.get(&reg3_name) {
                return Ok((reg_ref, reg3_name.clone()));
            } else {
                return Err(RizeError {
                    type_: RizeErrorType::RegisterRead(format!(
                        "Cannot get a register named \"{}\"",
                        reg3_name
                    )),
                });
            }
        }
        ArgType::None => {
            if let ArgType::Register(reg1_name) = arg1 {
                if let Some(reg_ref) = registers.get(&reg1_name) {
                    return Ok((reg_ref, reg1_name.clone()));
                } else {
                    return Err(RizeError {
                        type_: RizeErrorType::RegisterRead(format!(
                            "Cannot get a register named \"{}\"",
                            reg1_name
                        )),
                    });
                }
            } else {
                Err(RizeError {
                    type_: RizeErrorType::Execute(
                        "Destination (arg1) must be a Register when arg3 is omitted.".to_string(),
                    ),
                })
            }
        }
        _ => {
            return Err(RizeError {
                type_: RizeErrorType::Decode(format!(
                    "Third argument (destination) must be a Register or omitted"
                )),
            })
        }
    }
}

pub fn mov(
    arg1: &ProgramArg, // Destination (Register or MemAddr)
    arg2: &ProgramArg, // Source (Register, Immediate, MemAddr)
    registers: &mut Registers,
    memory: &SystemMemory,
) -> Result<(), RizeError> {
    // Unwrap is OK cause arg2 has been validated in Decode stage
    let source_value = arg2.value.clone().unwrap();

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
            if let Some(byte) = memory.bytes.get(&(*dest_addr as usize)) {
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
                "MOV destination (arg1) must be Register or MemAddr.".to_string(),
            ),
        }),
    }
}

pub fn add(
    arg1: &ProgramArg,
    arg2: &ProgramArg,
    arg3_opt: &ProgramArg,
    registers: &mut Registers,
) -> Result<(), RizeError> {
    // Get arg1 and arg2 values. Unwrap is OK cause both have been validated in Decode stage
    let v1 = arg1.value.clone().unwrap();
    let v2 = arg2.value.clone().unwrap();

    // Ensure arg1 is a register (destination or source)
    if !matches!(arg1.arg_type, ArgType::Register(_)) {
        return Err(RizeError {
            type_: RizeErrorType::Execute(
                "ADD requires the first argument (arg1) to be a register.".to_string(),
            ),
        });
    }

    let result = &(v1.clone().add(v2.clone()));

    // Determine destination register using helper
    let (_dest_register, dest_name) =
        determine_destination_register_mut(registers, &arg1.arg_type, &arg3_opt.arg_type)?;

    // --- Set Flags ---
    // Zero Flag (fz): Set if result is 0
    registers
        .get(FLAG_ZERO)
        .ok_or_else(|| RizeError {
            type_: RizeErrorType::RegisterRead(format!("Flag register '{}' not found", FLAG_ZERO)),
        })?
        .write(DSB::Flag(result.clone() == DSB::from(0)))?;
    // Negative Flag (fn): Set if MSB of result is 1
    registers
        .get(FLAG_NEGATIVE)
        .ok_or_else(|| RizeError {
            type_: RizeErrorType::RegisterRead(format!(
                "Flag register '{}' not found",
                FLAG_NEGATIVE
            )),
        })?
        .write(DSB::Flag(result.clone() & 0x8000.into() != 0.into()))?; // Check MSB
    // Carry Flag (fc): Set if unsigned addition resulted in carry
    let carry = (v1.as_u128() + v2.as_u128()) > 0xFFFF;
    registers
        .get(FLAG_CARRY)
        .ok_or_else(|| RizeError {
            type_: RizeErrorType::RegisterRead(format!("Flag register '{}' not found", FLAG_CARRY)),
        })?
        .write(DSB::Flag(carry))?;
    // Overflow Flag (fo): Set if signed addition resulted in overflow
    let v1_sign = (v1.clone() >> 15.into()) & 1.into();
    let v2_sign = (v2.clone() >> 15.into()) & 1.into();
    let result_sign = (result.clone() >> 15.into()) & 1.into();
    let overflow = (v1_sign == v2_sign) && (result_sign != v1_sign);
    registers
        .get(FLAG_OVERFLOW)
        .ok_or_else(|| RizeError {
            type_: RizeErrorType::RegisterRead(format!(
                "Flag register '{}' not found",
                FLAG_OVERFLOW
            )),
        })?
        .write(DSB::Flag(overflow))?;
    // --- End Set Flags ---

    // Get register ref again (determine_... returns name now)
    if let Some(register) = registers.get(&dest_name) {
        register.write(result.clone())
    } else {
        Err(RizeError {
            type_: RizeErrorType::RegisterRead(format!(
                "Cannot get a register named \"{}\"",
                dest_name
            )),
        })
    }
}