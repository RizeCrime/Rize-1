use crate::types::{ArgType, ByteOperations, Registers, RizeError, RizeErrorType, SystemMemory, DSB};

/// Determines the value of an operand (Register, Immediate, or Memory Address).
pub fn get_operand_value(
    registers: &mut Registers,
    memory: &SystemMemory,
    arg: &ArgType,
) -> Result<DSB, RizeError> {
    match &arg {
        ArgType::Register(reg_name) => {
            // Unwrap, because no RizeErrorType expect RegisterGet error
            if let Some(register) = registers.get(&reg_name) {
                return register.read()
            }
            else {
                return Err(RizeError { type_: RizeErrorType::RegisterRead(format!("Cannot get a register named \"{}\"", reg_name)) })
            }
        }
        ArgType::Immediate(imm) => return Ok(DSB::U16(*imm)),
        ArgType::MemAddr(addr) => {
            if let Some(byte) = memory.bytes.get(&(*addr as usize)) {
                return byte.read();
            }
            else {
                return Err(RizeError { type_: RizeErrorType::MemoryRead(format!("No byte found at address {}", addr)) })
            }
        }
        ArgType::Symbol(sym) => return Err(RizeError {
            type_: RizeErrorType::Decode(format!(
                "Cannot use symbol '.{}' as an operand value.",
                sym
            ))
        }),
        ArgType::None | ArgType::Error => return Err(RizeError {
            type_: RizeErrorType::Decode("Invalid/None ArgType encountered where value operand expected.".to_string()),
        }),
    }
}

// /// Determines the destination register for instructions with an optional 3rd argument.
// /// Returns a mutable reference to the destination register.
// /// UPDATED: Also returns the name of the destination register as a String.
// fn determine_destination_register_mut<'a>(
//     registers: &'a mut Registers,
//     arg1: &ArgType,
//     arg3_opt: &Option<ArgType>,
// ) -> Result<(&'a mut Register, String), RizeError> {
//     match arg3_opt {
//         // If arg3 is provided and is a register
//         Some(ArgType::Register(reg3_name)) => {
//             let reg_ref = get_register_mut(registers, reg3_name)?;
//             Ok((reg_ref, reg3_name.clone()))
//         }
//         // If arg3 is None or a comment, use arg1 (which must be a register)
//         None | Some(ArgType::None) => {
//             if let ArgType::Register(reg1_name) = arg1 {
//                 let reg_ref = get_register_mut(registers, reg1_name)?;
//                 Ok((reg_ref, reg1_name.clone()))
//             } else {
//                 Err(RizeError {
//                     type_: RizeErrorType::Execute,
//                     message: "Destination (arg1) must be a Register when arg3 is omitted."
//                         .to_string(),
//                 })
//             }
//         }
//         // If arg3 is provided but is not a register
//         Some(_) => Err(RizeError {
//             type_: RizeErrorType::Execute,
//             message:
//                 "Third argument (destination) must be a Register or omitted."
//                     .to_string(),
//         }),
//     }
// }

// /// ### Parsing Rules
// ///
// /// Rules apply in Order, returning the first match.
// ///
// /// 0) if starts with '#'       -> Comment (Ignore)
// /// 1) if only characters       -> Register
// /// 2) if starts with '0x'      -> MemAddr
// /// 3) if is entirely digits    -> Immediate
// /// 4) if starts with '.'       -> Symbol
// fn parse_arg(arg: &str) -> ArgType {
//     if arg.is_empty() {
//         return ArgType::None;
//     }

//     // Rule 0: Comments
//     if arg.starts_with('#') {
//         return ArgType::None;
//     }

//     // Rule 1: Register
//     if arg.chars().all(|c| c.is_alphabetic()) {
//         return ArgType::Register(arg.to_string());
//     }

//     // Rule 2: Memory Address (Hexadecimal)
//     if let Some(hex_val) = arg.strip_prefix("0x") {
//         if let Ok(addr) = u16::from_str_radix(hex_val, 16) {
//             return ArgType::MemAddr(addr);
//         }
//         return ArgType::Error;
//     }

//     // Rule 3: Immediate (Decimal)
//     if let Ok(imm) = arg.parse::<u16>() {
//         return ArgType::Immediate(imm);
//     }

//     // Rule 4: Symbol
//     if let Some(symbol_name) = arg.strip_prefix('.') {
//         if !symbol_name.is_empty()
//             && symbol_name.chars().all(char::is_alphabetic)
//         {
//             return ArgType::Symbol(symbol_name.to_string());
//         }
//         // If it starts with '.' but isn't a valid symbol format
//         return ArgType::Error;
//     }

//     // Default/Error if none of the above match
//     ArgType::Error
// }

// fn mov(
//     arg1: &ArgType, // Destination (Register or MemAddr)
//     arg2: &ArgType, // Source (Register, Immediate, MemAddr)
//     registers: &mut Registers,
//     memory: &mut Memory, // Needs mutable memory for MemAddr dest
// ) -> Result<(), RizeError> {
//     let source_value = get_operand_value(registers, memory, arg2)?;

//     match arg1 {
//         ArgType::Register(dest_reg_name) => {
//             let register = get_register_mut(registers, dest_reg_name)?;
//             // Use the new trait method
//             register.write_section_u16(source_value)
//         }
//         ArgType::MemAddr(dest_addr) => {
//             memory.write(*dest_addr, source_value) // write returns Result
//         }
//         _ => Err(RizeError {
//             type_: RizeErrorType::Execute,
//             message: "MOV destination (arg1) must be Register or MemAddr."
//                 .to_string(),
//         }),
//     }
// }

// fn add(
//     arg1: &ArgType,
//     arg2: &ArgType,
//     arg3_opt: &Option<ArgType>,
//     registers: &mut Registers,
// ) -> Result<(), RizeError> {
//     // Validate arg1 is a register and get its value
//     let v1 = get_operand_value(registers, &Memory::new(), arg1)?;
//     let v2 = get_operand_value(registers, &Memory::new(), arg2)?;

//     // Ensure arg1 is a register (destination or source)
//     if !matches!(arg1, ArgType::Register(_)) {
//         return Err(RizeError {
//             type_: RizeErrorType::Execute,
//             message: "ADD requires the first argument (arg1) to be a register."
//                 .to_string(),
//         });
//     }

//     // Perform addition using wrapping arithmetic
//     let result = v1.wrapping_add(v2);

//     // Determine destination register using helper
//     let (_dest_register, dest_name) =
//         determine_destination_register_mut(registers, arg1, arg3_opt)?;

//     // --- Set Flags ---
//     // Zero Flag (fz): Set if result is 0
//     registers
//         .get(FLAG_ZERO)
//         .ok_or_else(|| RizeError {
//             type_: RizeErrorType::RegisterRead,
//             message: format!("Flag register '{}' not found", FLAG_ZERO),
//         })?
//         .write_bool(result == 0)?;
//     // Negative Flag (fn): Set if MSB of result is 1
//     registers
//         .get(FLAG_NEGATIVE)
//         .ok_or_else(|| RizeError {
//             type_: RizeErrorType::RegisterRead,
//             message: format!("Flag register '{}' not found", FLAG_NEGATIVE),
//         })?
//         .write_bool(result & 0x8000 != 0)?; // Check MSB
//                                             // Carry Flag (fc): Set if unsigned addition resulted in carry
//     let carry = (v1 as u32 + v2 as u32) > 0xFFFF;
//     registers
//         .get(FLAG_CARRY)
//         .ok_or_else(|| RizeError {
//             type_: RizeErrorType::RegisterRead,
//             message: format!("Flag register '{}' not found", FLAG_CARRY),
//         })?
//         .write_bool(carry)?;
//     // Overflow Flag (fo): Set if signed addition resulted in overflow
//     let v1_sign = (v1 >> 15) & 1;
//     let v2_sign = (v2 >> 15) & 1;
//     let result_sign = (result >> 15) & 1;
//     let overflow = (v1_sign == v2_sign) && (result_sign != v1_sign);
//     registers
//         .get(FLAG_OVERFLOW)
//         .ok_or_else(|| RizeError {
//             type_: RizeErrorType::RegisterRead,
//             message: format!("Flag register '{}' not found", FLAG_OVERFLOW),
//         })?
//         .write_bool(overflow)?;
//     // --- End Set Flags ---

//     // Get register ref again (determine_... returns name now)
//     let dest_register = get_register_mut(registers, &dest_name)?;
//     // Use section-aware trait method
//     dest_register.write_section_u16(result)
// }

// fn sub(
//     arg1: &ArgType,
//     arg2: &ArgType,
//     arg3_opt: &Option<ArgType>,
//     registers: &mut Registers,
//     r_memory: &Memory,
// ) -> Result<(), RizeError> {
//     // Validate arg1 is a register and get its value
//     let v1 = get_operand_value(registers, r_memory, arg1)?;
//     // Get the value of arg2 (can be Register or Immediate)
//     let v2 = get_operand_value(registers, r_memory, arg2)?;

//     // Ensure arg1 is a register (destination or source)
//     if !matches!(arg1, ArgType::Register(_)) {
//         return Err(RizeError {
//             type_: RizeErrorType::Execute,
//             message: "SUB requires the first argument (arg1) to be a register."
//                 .to_string(),
//         });
//     }

//     // Perform subtraction using wrapping arithmetic
//     let result = v1.wrapping_sub(v2);

//     // Determine destination register using helper
//     let (_dest_register, dest_name) =
//         determine_destination_register_mut(registers, arg1, arg3_opt)?;

//     // --- Set Flags ---
//     // Zero Flag (fz): Set if result is 0
//     registers
//         .get(FLAG_ZERO)
//         .ok_or_else(|| RizeError {
//             type_: RizeErrorType::RegisterRead,
//             message: format!("Flag register '{}' not found", FLAG_ZERO),
//         })?
//         .write_bool(result == 0)?;
//     // Negative Flag (fn): Set if MSB of result is 1
//     registers
//         .get(FLAG_NEGATIVE)
//         .ok_or_else(|| RizeError {
//             type_: RizeErrorType::RegisterRead,
//             message: format!("Flag register '{}' not found", FLAG_NEGATIVE),
//         })?
//         .write_bool(result & 0x8000 != 0)?; // Check MSB
//                                             // Carry Flag (fc): Set if unsigned subtraction resulted in borrow (v1 < v2)
//     let borrow = v1 < v2;
//     registers
//         .get(FLAG_CARRY) // Often called Borrow flag in subtraction context
//         .ok_or_else(|| RizeError {
//             type_: RizeErrorType::RegisterRead,
//             message: format!("Flag register '{}' not found", FLAG_CARRY),
//         })?
//         .write_bool(borrow)?;
//     // Overflow Flag (fo): Set if signed subtraction resulted in overflow
//     let v1_sign = (v1 >> 15) & 1;
//     let v2_sign = (v2 >> 15) & 1;
//     let result_sign = (result >> 15) & 1;
//     let overflow = (v1_sign != v2_sign) && (result_sign != v1_sign);
//     registers
//         .get(FLAG_OVERFLOW)
//         .ok_or_else(|| RizeError {
//             type_: RizeErrorType::RegisterRead,
//             message: format!("Flag register '{}' not found", FLAG_OVERFLOW),
//         })?
//         .write_bool(overflow)?;
//     // --- End Set Flags ---

//     // Get register ref again
//     let dest_register = get_register_mut(registers, &dest_name)?;
//     // Use section-aware trait method
//     dest_register.write_section_u16(result)
// }

// fn st(registers: &mut Registers, memory: &mut Memory) -> Result<(), RizeError> {
//     let address_reg = get_register_mut(registers, "mar")?;
//     let address = address_reg.read_section_u16()?;
//     let data_reg = get_register_mut(registers, "mdr")?;
//     let data = data_reg.read_section_u16()?;
//     memory.write(address, data)
// }

// fn ld(registers: &mut Registers, memory: &Memory) -> Result<(), RizeError> {
//     let address_reg = get_register_mut(registers, "mar")?;
//     let address = address_reg.read_section_u16()?;
//     let data = memory.read(address)?;
//     let data_reg = get_register_mut(registers, "mdr")?;
//     data_reg.write_section_u16(data)
// }

// fn and(
//     arg1: &ArgType,
//     arg2: &ArgType,
//     arg3_opt: &Option<ArgType>,
//     registers: &mut Registers,
// ) -> Result<(), RizeError> {
//     let v1 = get_operand_value(registers, &Memory::new(), arg1)?;
//     let v2 = get_operand_value(registers, &Memory::new(), arg2)?;

//     if !matches!(arg1, ArgType::Register(_))
//         || !matches!(arg2, ArgType::Register(_))
//     {
//         return Err(RizeError {
//             type_: RizeErrorType::Execute,
//             message: "AND requires register operands (arg1, arg2).".to_string(),
//         });
//     }

//     let result = v1 & v2;

//     let (_dest_register, dest_name) =
//         determine_destination_register_mut(registers, arg1, arg3_opt)?;
//     // Get register ref again
//     let dest_register = get_register_mut(registers, &dest_name)?;
//     // Use section-aware trait method
//     dest_register.write_section_u16(result)
// }

// fn or(
//     arg1: &ArgType,
//     arg2: &ArgType,
//     arg3_opt: &Option<ArgType>,
//     registers: &mut Registers,
// ) -> Result<(), RizeError> {
//     let v1 = get_operand_value(registers, &Memory::new(), arg1)?;
//     let v2 = get_operand_value(registers, &Memory::new(), arg2)?;

//     if !matches!(arg1, ArgType::Register(_))
//         || !matches!(arg2, ArgType::Register(_))
//     {
//         return Err(RizeError {
//             type_: RizeErrorType::Execute,
//             message: "OR requires register operands (arg1, arg2).".to_string(),
//         });
//     }

//     let result = v1 | v2;

//     let (_dest_register, dest_name) =
//         determine_destination_register_mut(registers, arg1, arg3_opt)?;
//     // Get register ref again
//     let dest_register = get_register_mut(registers, &dest_name)?;
//     // Use section-aware trait method
//     dest_register.write_section_u16(result)
// }

// fn xor(
//     arg1: &ArgType,
//     arg2: &ArgType,
//     arg3_opt: &Option<ArgType>,
//     registers: &mut Registers,
// ) -> Result<(), RizeError> {
//     let v1 = get_operand_value(registers, &Memory::new(), arg1)?;
//     let v2 = get_operand_value(registers, &Memory::new(), arg2)?;

//     if !matches!(arg1, ArgType::Register(_))
//         || !matches!(arg2, ArgType::Register(_))
//     {
//         return Err(RizeError {
//             type_: RizeErrorType::Execute,
//             message: "XOR requires register operands (arg1, arg2).".to_string(),
//         });
//     }

//     let result = v1 ^ v2;

//     let (_dest_register, dest_name) =
//         determine_destination_register_mut(registers, arg1, arg3_opt)?;
//     // Get register ref again
//     let dest_register = get_register_mut(registers, &dest_name)?;
//     // Use section-aware trait method
//     dest_register.write_section_u16(result)
// }

// fn not(arg1: &ArgType, registers: &mut Registers) -> Result<(), RizeError> {
//     if let ArgType::Register(reg_name) = arg1 {
//         let register = get_register_mut(registers, reg_name)?;
//         let v1 = register.read_section_u16()?;
//         let result = !v1;
//         register.write_section_u16(result)
//     } else {
//         Err(RizeError {
//             type_: RizeErrorType::Execute,
//             message: "NOT requires a Register operand (arg1).".to_string(),
//         })
//     }
// }

// fn shl(
//     arg1: &ArgType, // Target Register
//     arg2: &ArgType, // Amount Immediate (Optional, defaults to 1)
//     registers: &mut Registers,
// ) -> Result<(), RizeError> {
//     if let ArgType::Register(target_reg_name) = arg1 {
//         let amount = match arg2 {
//             ArgType::Immediate(imm) => *imm,
//             ArgType::None => 1, // Default shift amount
//             _ => {
//                 return Err(RizeError {
//                     type_: RizeErrorType::Execute,
//                     message: "SHL amount (arg2) must be Immediate or omitted."
//                         .to_string(),
//                 })
//             }
//         };

//         let target_register = get_register_mut(registers, target_reg_name)?;
//         let value = target_register.read_section_u16()?;
//         let result = value << amount;
//         target_register.write_section_u16(result)
//     } else {
//         Err(RizeError {
//             type_: RizeErrorType::Execute,
//             message: "SHL target (arg1) must be a Register.".to_string(),
//         })
//     }
// }

// fn shr(
//     arg1: &ArgType, // Target Register
//     arg2: &ArgType, // Amount Immediate (Optional, defaults to 1)
//     registers: &mut Registers,
// ) -> Result<(), RizeError> {
//     if let ArgType::Register(target_reg_name) = arg1 {
//         let amount = match arg2 {
//             ArgType::Immediate(imm) => *imm,
//             ArgType::None => 1,
//             _ => {
//                 return Err(RizeError {
//                     type_: RizeErrorType::Execute,
//                     message: "SHR amount (arg2) must be Immediate or omitted."
//                         .to_string(),
//                 })
//             }
//         };

//         let target_register = get_register_mut(registers, target_reg_name)?;
//         let value = target_register.read_section_u16()?;
//         let result = value >> amount;
//         target_register.write_section_u16(result)
//     } else {
//         Err(RizeError {
//             type_: RizeErrorType::Execute,
//             message: "SHR target (arg1) must be a Register.".to_string(),
//         })
//     }
// }

// fn wdm(
//     arg1: &ArgType,
//     arg2: &ArgType,
//     arg3: &ArgType,
//     mut r_display_memory: ResMut<DisplayMemory>,
//     registers: &mut Registers,
//     memory: &Memory, // Added memory
// ) -> Result<(), RizeError> {
//     let val1: u16 = get_operand_value(registers, memory, arg1)?;
//     let val2: u16 = get_operand_value(registers, memory, arg2)?;
//     let val3: u16 = get_operand_value(registers, memory, arg3)?;

//     // info!("Val3: {:#016b}", val3);

//     let red: u8 = (val1 >> 8) as u8;
//     let green: u8 = (val1 & 0xFF) as u8;

//     let blue: u8 = (val2 >> 8) as u8;
//     let alpha: u8 = (val2 & 0xFF) as u8;

//     let x: u8 = (val3 >> 8) as u8;
//     let y: u8 = (val3 & 0xFF) as u8;

//     // info(format!("x: {x}, y: {y}"));

//     let color = [red, green, blue, alpha];

//     r_display_memory.set_pixel(x, y, color)
// }

// fn mul(
//     arg1: &ArgType,
//     arg2: &ArgType,
//     arg3_opt: &Option<ArgType>,
//     registers: &mut Registers,
//     r_memory: &Memory,
// ) -> Result<(), RizeError> {
//     // Validate arg1 is a register and get its value
//     let v1 = get_operand_value(registers, r_memory, arg1)?;
//     // Get the value of arg2 (can be Register or Immediate)
//     let v2 = get_operand_value(registers, r_memory, arg2)?;

//     // Ensure arg1 is a register (destination or source)
//     if !matches!(arg1, ArgType::Register(_)) {
//         return Err(RizeError {
//             type_: RizeErrorType::Execute,
//             message: "MUL requires the first argument (arg1) to be a register."
//                 .to_string(),
//         });
//     }

//     // Perform multiplication using wrapping arithmetic
//     let result = v1.wrapping_mul(v2);

//     // Determine destination register using helper
//     let (_dest_register, dest_name) =
//         determine_destination_register_mut(registers, arg1, arg3_opt)?;

//     // --- Set Flags ---
//     // Zero Flag (fz): Set if result is 0
//     registers
//         .get(FLAG_ZERO)
//         .ok_or_else(|| RizeError {
//             type_: RizeErrorType::RegisterRead,
//             message: format!("Flag register '{}' not found", FLAG_ZERO),
//         })?
//         .write_bool(result == 0)?;
//     // Negative Flag (fn): Set if MSB of result is 1
//     registers
//         .get(FLAG_NEGATIVE)
//         .ok_or_else(|| RizeError {
//             type_: RizeErrorType::RegisterRead,
//             message: format!("Flag register '{}' not found", FLAG_NEGATIVE),
//         })?
//         .write_bool(result & 0x8000 != 0)?; // Check MSB
//                                             // Carry Flag (fc): Set if the product exceeds 16 bits
//     let carry = (v1 as u32 * v2 as u32) > 0xFFFF;
//     registers
//         .get(FLAG_CARRY)
//         .ok_or_else(|| RizeError {
//             type_: RizeErrorType::RegisterRead,
//             message: format!("Flag register '{}' not found", FLAG_CARRY),
//         })?
//         .write_bool(carry)?;
//     // Overflow Flag (fo): Set if signed multiplication resulted in overflow
//     // Overflow occurs when the result cannot be correctly represented in 16 bits
//     let v1_signed = v1 as i16;
//     let v2_signed = v2 as i16;
//     let result_i32 = v1_signed as i32 * v2_signed as i32;
//     let overflow = result_i32 < i16::MIN as i32 || result_i32 > i16::MAX as i32;
//     registers
//         .get(FLAG_OVERFLOW)
//         .ok_or_else(|| RizeError {
//             type_: RizeErrorType::RegisterRead,
//             message: format!("Flag register '{}' not found", FLAG_OVERFLOW),
//         })?
//         .write_bool(overflow)?;
//     // --- End Set Flags ---

//     // Get register ref again
//     let dest_register = get_register_mut(registers, &dest_name)?;
//     // Use section-aware trait method
//     dest_register.write_section_u16(result)
// }

// fn div(
//     arg1: &ArgType,
//     arg2: &ArgType,
//     arg3_opt: &Option<ArgType>,
//     registers: &mut Registers,
//     r_memory: &Memory,
// ) -> Result<(), RizeError> {
//     // Validate arg1 is a register and get its value
//     let v1 = get_operand_value(registers, r_memory, arg1)?;
//     // Get the value of arg2 (can be Register or Immediate)
//     let v2 = get_operand_value(registers, r_memory, arg2)?;

//     // Ensure arg1 is a register (destination or source)
//     if !matches!(arg1, ArgType::Register(_)) {
//         return Err(RizeError {
//             type_: RizeErrorType::Execute,
//             message: "DIV requires the first argument (arg1) to be a register."
//                 .to_string(),
//         });
//     }

//     // Check for division by zero
//     if v2 == 0 {
//         return Err(RizeError {
//             type_: RizeErrorType::Execute,
//             message: "Division by zero".to_string(),
//         });
//     }

//     // Perform division
//     let result = v1.wrapping_div(v2);

//     // Determine destination register using helper
//     let (_dest_register, dest_name) =
//         determine_destination_register_mut(registers, arg1, arg3_opt)?;

//     // --- Set Flags ---
//     // Zero Flag (fz): Set if result is 0
//     registers
//         .get(FLAG_ZERO)
//         .ok_or_else(|| RizeError {
//             type_: RizeErrorType::RegisterRead,
//             message: format!("Flag register '{}' not found", FLAG_ZERO),
//         })?
//         .write_bool(result == 0)?;
//     // Negative Flag (fn): Set if MSB of result is 1
//     registers
//         .get(FLAG_NEGATIVE)
//         .ok_or_else(|| RizeError {
//             type_: RizeErrorType::RegisterRead,
//             message: format!("Flag register '{}' not found", FLAG_NEGATIVE),
//         })?
//         .write_bool(result & 0x8000 != 0)?; // Check MSB
//                                             // Carry Flag (fc): Set if there is a remainder
//     let remainder = v1 % v2;
//     registers
//         .get(FLAG_CARRY)
//         .ok_or_else(|| RizeError {
//             type_: RizeErrorType::RegisterRead,
//             message: format!("Flag register '{}' not found", FLAG_CARRY),
//         })?
//         .write_bool(remainder != 0)?;
//     // Overflow Flag (fo): Set if signed division resulted in overflow
//     // Overflow can only occur in signed division when dividing INT_MIN by -1
//     let v1_signed = v1 as i16;
//     let v2_signed = v2 as i16;
//     let overflow = v1_signed == i16::MIN && v2_signed == -1;
//     registers
//         .get(FLAG_OVERFLOW)
//         .ok_or_else(|| RizeError {
//             type_: RizeErrorType::RegisterRead,
//             message: format!("Flag register '{}' not found", FLAG_OVERFLOW),
//         })?
//         .write_bool(overflow)?;
//     // --- End Set Flags ---

//     // Get register ref again
//     let dest_register = get_register_mut(registers, &dest_name)?;
//     // Use section-aware trait method
//     dest_register.write_section_u16(result)
// }