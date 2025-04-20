use std::collections::HashMap;
use std::fs::{self, File};
use std::io::BufRead;
use std::path::PathBuf;
use std::str::{FromStr, Lines};

use bevy::prelude::*;
use bevy::tasks::futures_lite::stream::Pending;
use bevy_inspector_egui::prelude::*;
use display::RizeOneDisplay;

use super::*;
use crate::*;

mod display;
pub use display::*;

#[derive(Resource, Default, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct AzmPrograms(pub Vec<(PathBuf, String)>);

#[derive(Resource, Default, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct ActiveProgram {
    pub auto_step: bool,
    pub path: PathBuf,
    pub file_stem: String,
    pub contents: String,
    pub line: usize,
    pub symbols: HashMap<String, usize>,
    pub raw_opcode: String,
    pub opcode: OpCode,
    pub arg1: ProgramArg,
    pub arg2: ProgramArg,
    pub arg3: ProgramArg,
}

#[derive(Resource, Default, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct ProgramArg {
    pub raw: String,
    pub parsed: ArgType,
}

#[derive(
    Resource, Default, Reflect, InspectorOptions, Clone, Eq, PartialEq,
)]
#[reflect(Resource, InspectorOptions)]
pub enum ArgType {
    #[default]
    None,
    Error,
    Register(String),
    MemAddr(u16),
    Immediate(u16),
    Symbol(String),
}

#[derive(Resource)]
pub struct FileCheckTimer(Timer);

pub struct RizeOneInterpreter;

impl Plugin for RizeOneInterpreter {
    fn build(&self, app: &mut App) {
        app.insert_resource(AzmPrograms::default());
        app.insert_resource(ActiveProgram::default());
        app.insert_resource(FileCheckTimer(Timer::from_seconds(
            0.25,
            TimerMode::Repeating,
        )));

        app.register_type::<AzmPrograms>();
        app.register_type::<ActiveProgram>();

        #[cfg(debug_assertions)]
        app.add_plugins(ResourceInspectorPlugin::<ActiveProgram>::default());

        app.add_plugins(RizeOneDisplay);

        app.add_systems(Update, check_azm_programs);

        app.add_systems(
            OnEnter(CpuCycleStage::Fetch),
            (tick_cpu, fetch).chain(),
        );
        app.add_systems(
            OnEnter(CpuCycleStage::Decode),
            (tick_cpu, decode).chain(),
        );
        app.add_systems(
            OnEnter(CpuCycleStage::Execute),
            (tick_cpu, execute).chain(),
        );
    }
}

pub fn update_program_counter(
    r_program: Res<ActiveProgram>,
    mut r_registers: ResMut<Registers>,
) {
    let pc: &mut Register = r_registers.get("pc").unwrap();
    let value: usize = r_program.line;

    pc.store_immediate(value).unwrap();
}

pub fn check_azm_programs(
    mut r_programs: ResMut<AzmPrograms>,
    time: Res<Time>,
    mut timer: ResMut<FileCheckTimer>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let azzembly_dir = AZZEMBLY_DIR;
    // debug!("Checking for .azm programs in {}", azzembly_dir);

    let entries = match fs::read_dir(azzembly_dir) {
        Ok(entries) => entries,
        Err(e) => {
            error!("Error reading directory {}: {}", azzembly_dir, e);
            return;
        }
    };

    for entry_result in entries {
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(e) => {
                error!("Error reading directory entry: {}", e);
                continue; // Skip this entry and continue with the next
            }
        };

        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        if path.extension().map_or(false, |ext| ext != "azm") {
            continue;
        }

        // Check if the program already exists
        if r_programs.0.iter().any(|(p, _)| p == &path) {
            continue;
        }

        // If all checks pass, add the new program
        info!("Found new .azm program: {:?}", path);
        let file_stem = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        r_programs.0.push((path.clone(), file_stem));
    }
}

pub fn tick_cpu(
    s_current_stage: Res<State<CpuCycleStage>>,
    mut s_next_stage: ResMut<NextState<CpuCycleStage>>,
    mut r_program: ResMut<ActiveProgram>,
) {
    if !r_program.auto_step {
        return;
    }

    match s_current_stage.get() {
        CpuCycleStage::Fetch => {
            s_next_stage.set(CpuCycleStage::Decode);
        }
        CpuCycleStage::Decode => {
            s_next_stage.set(CpuCycleStage::Execute);
        }
        CpuCycleStage::Execute => {
            s_next_stage.set(CpuCycleStage::Fetch);
        }
        CpuCycleStage::Halt => {
            s_next_stage.set(CpuCycleStage::Halt);
            r_program.auto_step = false;
        }
        _ => {}
    }
}

/// -------------- ///
/// Update Systems ///
/// -------------- ///

pub fn fetch(
    mut r_active_program: ResMut<ActiveProgram>,
    mut next_cpu_stage: ResMut<NextState<CpuCycleStage>>,
    mut r_registers: ResMut<Registers>,
) {
    let mut program = r_active_program.as_mut();

    // Create an iterator starting from the current line
    let mut lines_iter = program.contents.lines().skip(program.line);

    loop {
        if let Some(line_str) = lines_iter.next() {
            let trimmed_line = line_str.trim();

            // Check if the line is empty or a comment
            if trimmed_line.is_empty()
                || trimmed_line.starts_with('#')
                || trimmed_line.starts_with('.')
            {
                program.line += 1; // Increment line counter for the skipped line
                continue; // Try the next line
            }

            // Process the valid instruction line
            let parts: Vec<&str> = trimmed_line.split_whitespace().collect();
            program.raw_opcode =
                parts.get(0).copied().unwrap_or_default().to_string();
            program.arg1 = ProgramArg {
                raw: parts.get(1).copied().unwrap_or_default().to_string(),
                parsed: ArgType::None,
            };
            program.arg2 = ProgramArg {
                raw: parts.get(2).copied().unwrap_or_default().to_string(),
                parsed: ArgType::None,
            };
            program.arg3 = ProgramArg {
                raw: parts.get(3).copied().unwrap_or_default().to_string(),
                parsed: ArgType::None,
            };

            program.line += 1;
            break;
        } else {
            info!("End of program reached. Halting CPU.");
            next_cpu_stage.set(CpuCycleStage::Halt);

            program.raw_opcode = String::new(); // Clear fields
            program.opcode = OpCode::None;
            program.arg1 = ProgramArg::default();
            program.arg2 = ProgramArg::default();
            program.arg3 = ProgramArg::default();

            // program.line remains at the position *after* the last line
            break;
        }
    }

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
}

// 1) validate opcode via enum from String
// 2) parse args from String
pub fn decode(mut r_active_program: ResMut<ActiveProgram>) {
    let mut program = r_active_program.as_mut();

    program.opcode = OpCode::from_str(&program.raw_opcode).unwrap_or_default();

    program.arg1.parsed = parse_arg(&program.arg1.raw);
    program.arg2.parsed = parse_arg(&program.arg2.raw);
    program.arg3.parsed = parse_arg(&program.arg3.raw);
}

pub fn execute(
    mut r_active_program: ResMut<ActiveProgram>,
    mut r_registers: ResMut<Registers>,
    mut r_memory: ResMut<Memory>,
    mut next_cpu_stage: ResMut<NextState<CpuCycleStage>>,
    mut r_display_memory: ResMut<DisplayMemory>,
    r_images: Res<Assets<Image>>,
) {
    let program = r_active_program.as_mut();
    let registers = r_registers.as_mut();
    let memory = r_memory.as_mut();

    let execution_result = match program.opcode {
        OpCode::MOV => mov(
            &program.arg1.parsed,
            &program.arg2.parsed,
            registers,
            memory,
        ),
        OpCode::ADD => {
            let arg3_option = if program.arg3.raw.is_empty() {
                None
            } else {
                Some(program.arg3.parsed.clone())
            };
            add(
                &program.arg1.parsed,
                &program.arg2.parsed,
                &arg3_option,
                registers,
            )
        }
        OpCode::SUB => {
            let arg3_option = if program.arg3.raw.is_empty() {
                None
            } else {
                Some(program.arg3.parsed.clone())
            };
            sub(
                &program.arg1.parsed,
                &program.arg2.parsed,
                &arg3_option,
                registers,
            )
        }
        OpCode::ST => st(registers, memory),
        OpCode::LD => ld(registers, memory),
        OpCode::AND => {
            let arg3_option = if program.arg3.raw.is_empty() {
                None
            } else {
                Some(program.arg3.parsed.clone())
            };
            and(
                &program.arg1.parsed,
                &program.arg2.parsed,
                &arg3_option,
                registers,
            )
        }
        OpCode::OR => {
            let arg3_option = if program.arg3.raw.is_empty() {
                None
            } else {
                Some(program.arg3.parsed.clone())
            };
            or(
                &program.arg1.parsed,
                &program.arg2.parsed,
                &arg3_option,
                registers,
            )
        }
        OpCode::XOR => {
            let arg3_option = if program.arg3.raw.is_empty() {
                None
            } else {
                Some(program.arg3.parsed.clone())
            };
            xor(
                &program.arg1.parsed,
                &program.arg2.parsed,
                &arg3_option,
                registers,
            )
        }
        OpCode::NOT => not(&program.arg1.parsed, registers),
        OpCode::SHL => {
            shl(&program.arg1.parsed, &program.arg2.parsed, registers)
        }
        OpCode::SHR => {
            shr(&program.arg1.parsed, &program.arg2.parsed, registers)
        }
        OpCode::HALT => {
            info!("Halting CPU!");
            next_cpu_stage.set(CpuCycleStage::Halt);
            Ok(())
        }
        OpCode::WDM => wdm(
            &program.arg1.parsed,
            &program.arg2.parsed,
            &program.arg3.parsed,
            r_display_memory,
            registers,
            memory,
        ),
        OpCode::JMP => {
            let target_symbol =
                program.arg1.raw.strip_prefix('.').unwrap_or_default();
            let target_line = program
                .symbols
                .get(target_symbol)
                .copied()
                .unwrap_or_default();
            if target_line == 0 {
                Err(RizeError {
                    type_: RizeErrorType::Execute,
                    message: format!(
                        "JMP target symbol '.{}' not found.",
                        target_symbol
                    ),
                })
            } else {
                program.line = target_line;
                // Also update PC register
                write_register_u16(registers, "pc", target_line as u16)
            }
        }
        OpCode::JIZ => {
            // Read flag, handling Result
            match read_register_u16(registers, "fz") {
                Ok(zero_flag) => {
                    if zero_flag == 1 {
                        // Jump logic
                        let target_symbol = program
                            .arg1
                            .raw
                            .strip_prefix('.')
                            .unwrap_or_default();
                        let target_line = program
                            .symbols
                            .get(target_symbol)
                            .copied()
                            .unwrap_or_default();
                        if target_line == 0 {
                            Err(RizeError {
                                type_: RizeErrorType::Execute,
                                message: format!(
                                    "JIZ target symbol '.{}' not found.",
                                    target_symbol
                                ),
                            })
                        } else {
                            program.line = target_line;
                            write_register_u16(
                                registers,
                                "pc",
                                target_line as u16,
                            )
                        }
                    } else {
                        // Flag is zero, don't jump
                        Ok(())
                    }
                }
                Err(e) => Err(e), // Propagate flag read error
            }
        }
        OpCode::JIN => {
            // Read flag, handling Result
            match read_register_u16(registers, "fn") {
                Ok(negative_flag) => {
                    if negative_flag == 1 {
                        // Jump logic
                        let target_symbol = program
                            .arg1
                            .raw
                            .strip_prefix('.')
                            .unwrap_or_default();
                        let target_line = program
                            .symbols
                            .get(target_symbol)
                            .copied()
                            .unwrap_or_default();
                        if target_line == 0 {
                            Err(RizeError {
                                type_: RizeErrorType::Execute,
                                message: format!(
                                    "JIN target symbol '.{}' not found.",
                                    target_symbol
                                ),
                            })
                        } else {
                            program.line = target_line;
                            write_register_u16(
                                registers,
                                "pc",
                                target_line as u16,
                            )
                        }
                    } else {
                        // Flag is zero, don't jump
                        Ok(())
                    }
                }
                Err(e) => Err(e), // Propagate flag read error
            }
        }
        _ => {
            warn!("OpCode {:?} not yet implemented!", program.opcode);
            Err(RizeError {
                type_: RizeErrorType::Execute,
                message: format!("OpCode {:?} not implemented", program.opcode),
            })
        }
    };

    // Handle the result of the execution
    if let Err(e) = execution_result {
        error!(
            "Execution Error ({:?}): {} (Op: {}, Args: '{}', '{}', '{}')",
            e.type_,
            e.message,
            program.raw_opcode,
            program.arg1.raw,
            program.arg2.raw,
            program.arg3.raw
        );
        // Potentially set a CPU Halted state here in the future
    }
}

/// ---------------- ///
/// Helper Functions ///
/// ---------------- ///

/// Gets a mutable reference to a register by name.
fn get_register_mut<'a>(
    registers: &'a mut Registers,
    reg_name: &str,
) -> Result<&'a mut Register, RizeError> {
    registers.get(reg_name).ok_or_else(|| RizeError {
        type_: RizeErrorType::RegisterRead, // Or RegisterWrite?
        message: format!("Register '{}' not found!", reg_name),
    })
}

/// Reads the u16 value from a register, respecting its current section setting.
fn read_register_u16(
    registers: &mut Registers,
    reg_name: &str,
) -> Result<u16, RizeError> {
    let register = get_register_mut(registers, reg_name)?;
    let bits = match register.section {
        'a' => register.read(),
        'b' => register.read_lower_half(),
        'c' => register.read_lower_quarter(),
        'd' => register.read_lower_eigth(),
        _ => {
            return Err(RizeError {
                type_: RizeErrorType::RegisterRead,
                message: format!(
                    "Invalid section '{}' found in register '{}' during read.",
                    register.section, reg_name
                ),
            })
        }
    } // This returns Result<Vec<i8>, &'static str>
    .map_err(|e| RizeError {
        // Map the trait error to RizeError
        type_: RizeErrorType::RegisterRead,
        message: format!(
            "Failed to read section '{}' from register {}: {}",
            register.section, reg_name, e
        ),
    })?;

    Ok(bits_to_u16(&bits))
}

/// Writes a u16 value to a register, respecting its current section setting.
fn write_register_u16(
    registers: &mut Registers,
    reg_name: &str,
    value: u16,
) -> Result<(), RizeError> {
    let register = get_register_mut(registers, reg_name)?;
    match register.section {
        'a' => register.store_immediate(value as usize),
        'b' => {
            let bits = u16_to_bits(value, CPU_BITTAGE / 2);
            register.write_lower_half(bits)
        }
        'c' => {
            let bits = u16_to_bits(value, CPU_BITTAGE / 4);
            register.write_lower_quarter(bits)
        }
        'd' => {
            let bits = u16_to_bits(value, CPU_BITTAGE / 8);
            register.write_lower_eigth(bits)
        }
        _ => Err(RizeError {
            type_: RizeErrorType::RegisterWrite,
            message: format!(
                "Invalid section '{}' found in register '{}' during write.",
                register.section, reg_name
            ),
        }),
    }
}

/// Converts a slice of bits (i8) into a u16, zero-extending if necessary.
/// Assumes MSB is at index 0.
fn bits_to_u16(bits: &[i8]) -> u16 {
    let mut value: u16 = 0;
    let len = bits.len();
    let start_bit_index = CPU_BITTAGE.saturating_sub(len); // Target bit index in u16
    debug!(
        "bits_to_u16: input_len={}, start_idx={}, input_bits={:?}",
        len, start_bit_index, bits
    );

    for (i, bit) in bits.iter().enumerate() {
        if *bit == 1 {
            let bit_pos = CPU_BITTAGE - 1 - (start_bit_index + i);
            value |= 1 << bit_pos;
        }
    }
    value
}

/// Converts the lower `num_bits` of a u16 into a Vec<i8>.
/// MSB will be at index 0.
fn u16_to_bits(value: u16, num_bits: usize) -> Vec<i8> {
    let mut bits = vec![0i8; num_bits];
    let start_bit_index_u16 = CPU_BITTAGE.saturating_sub(num_bits);
    debug!(
        "u16_to_bits: input_val=0x{:04X}, num_bits={}, start_idx_u16={}",
        value, num_bits, start_bit_index_u16
    );

    for i in 0..num_bits {
        // Corresponding bit index in the full u16 (from the left/MSB)
        let u16_idx = start_bit_index_u16 + i;
        // Bit position from the right (LSB=0) in the u16
        let bit_pos_from_lsb = CPU_BITTAGE - 1 - u16_idx;

        if (value >> bit_pos_from_lsb) & 1 == 1 {
            bits[i] = 1;
        }
    }
    debug!("u16_to_bits: output_bits={:?}", bits);
    bits
}

/// Determines the value of an operand (Register, Immediate, or Memory Address).
/// Reads section-aware for registers.
fn get_operand_value(
    registers: &mut Registers,
    memory: &Memory,
    arg: &ArgType,
) -> Result<u16, RizeError> {
    match arg {
        ArgType::Register(reg_name) => {
            // Now uses the section-aware read
            read_register_u16(registers, reg_name)
        }
        ArgType::Immediate(imm) => Ok(*imm),
        ArgType::MemAddr(addr) => {
            memory.read(*addr) // read already returns Result<u16, RizeError>
        }
        ArgType::Symbol(sym) => Err(RizeError {
            type_: RizeErrorType::Decode, // Or Execute?
            message: format!(
                "Cannot use symbol '.{}' as an operand value.",
                sym
            ),
        }),
        ArgType::None | ArgType::Error => Err(RizeError {
            type_: RizeErrorType::Decode, // Or Execute?
            message:
                "Invalid/None ArgType encountered where value operand expected."
                    .to_string(),
        }),
    }
}

/// Determines the destination register for instructions with an optional 3rd argument.
/// Returns a mutable reference to the destination register.
/// UPDATED: Also returns the name of the destination register as a String.
fn determine_destination_register_mut<'a>(
    registers: &'a mut Registers,
    arg1: &ArgType,
    arg3_opt: &Option<ArgType>,
) -> Result<(&'a mut Register, String), RizeError> {
    match arg3_opt {
        // If arg3 is provided and is a register
        Some(ArgType::Register(reg3_name)) => {
            let reg_ref = get_register_mut(registers, reg3_name)?;
            Ok((reg_ref, reg3_name.clone()))
        }
        // If arg3 is omitted, use arg1 (which must be a register)
        None => {
            if let ArgType::Register(reg1_name) = arg1 {
                let reg_ref = get_register_mut(registers, reg1_name)?;
                Ok((reg_ref, reg1_name.clone()))
            } else {
                Err(RizeError {
                    type_: RizeErrorType::Execute,
                    message: "Destination (arg1) must be a Register when arg3 is omitted."
                        .to_string(),
                })
            }
        }
        // If arg3 is provided but is not a register
        Some(_) => Err(RizeError {
            type_: RizeErrorType::Execute,
            message:
                "Third argument (destination) must be a Register or omitted."
                    .to_string(),
        }),
    }
}

/// ### Parsing Rules
///
/// Rules apply in Order, returning the first match.
///
/// 1) if only characters       -> Register
/// 2) if starts with '0x'      -> MemAddr
/// 3) if is entirely digits    -> Immediate
/// 4) if starts with '.'       -> Symbol
fn parse_arg(arg: &str) -> ArgType {
    if arg.is_empty() {
        return ArgType::None;
    }

    // Rule 1: Register
    if arg.chars().all(|c| c.is_alphabetic()) {
        return ArgType::Register(arg.to_string());
    }

    // Rule 2: Memory Address (Hexadecimal)
    if let Some(hex_val) = arg.strip_prefix("0x") {
        if let Ok(addr) = u16::from_str_radix(hex_val, 16) {
            return ArgType::MemAddr(addr);
        }
        return ArgType::Error;
    }

    // Rule 3: Immediate (Decimal)
    if let Ok(imm) = arg.parse::<u16>() {
        return ArgType::Immediate(imm);
    }

    // Rule 4: Symbol
    if let Some(symbol_name) = arg.strip_prefix('.') {
        if !symbol_name.is_empty()
            && symbol_name.chars().all(char::is_alphabetic)
        {
            return ArgType::Symbol(symbol_name.to_string());
        }
        // If it starts with '.' but isn't a valid symbol format
        return ArgType::Error;
    }

    // Default/Error if none of the above match
    ArgType::Error
}

fn mov(
    arg1: &ArgType, // Destination (Register or MemAddr)
    arg2: &ArgType, // Source (Register, Immediate, MemAddr)
    registers: &mut Registers,
    memory: &mut Memory, // Needs mutable memory for MemAddr dest
) -> Result<(), RizeError> {
    let source_value = get_operand_value(registers, memory, arg2)?;

    match arg1 {
        ArgType::Register(dest_reg_name) => {
            // TODO: Implement section-aware writing
            write_register_u16(registers, dest_reg_name, source_value)
        }
        ArgType::MemAddr(dest_addr) => {
            memory.write(*dest_addr, source_value) // write returns Result
        }
        _ => Err(RizeError {
            type_: RizeErrorType::Execute,
            message: "MOV destination (arg1) must be Register or MemAddr."
                .to_string(),
        }),
    }
}

fn add(
    arg1: &ArgType,
    arg2: &ArgType,
    arg3_opt: &Option<ArgType>,
    registers: &mut Registers,
) -> Result<(), RizeError> {
    // Validate arg1 and arg2 are registers and get their values
    let v1 = get_operand_value(registers, &Memory::new(), arg1)?; // Memory not needed here
    let v2 = get_operand_value(registers, &Memory::new(), arg2)?;

    if !matches!(arg1, ArgType::Register(_))
        || !matches!(arg2, ArgType::Register(_))
    {
        return Err(RizeError {
            type_: RizeErrorType::Execute,
            message: "ADD requires register operands (arg1, arg2).".to_string(),
        });
    }

    let result = v1.wrapping_add(v2);

    // Determine destination register using helper
    let (_dest_register, dest_name) =
        determine_destination_register_mut(registers, arg1, arg3_opt)?;

    // Use section-aware write helper
    write_register_u16(registers, &dest_name, result)
}

fn sub(
    arg1: &ArgType,
    arg2: &ArgType,
    arg3_opt: &Option<ArgType>,
    registers: &mut Registers,
) -> Result<(), RizeError> {
    let v1 = get_operand_value(registers, &Memory::new(), arg1)?;
    let v2 = get_operand_value(registers, &Memory::new(), arg2)?;

    if !matches!(arg1, ArgType::Register(_))
        || !matches!(arg2, ArgType::Register(_))
    {
        return Err(RizeError {
            type_: RizeErrorType::Execute,
            message: "SUB requires register operands (arg1, arg2).".to_string(),
        });
    }

    let result = v1.wrapping_sub(v2);

    let (_dest_register, dest_name) =
        determine_destination_register_mut(registers, arg1, arg3_opt)?;
    // Use section-aware write helper
    write_register_u16(registers, &dest_name, result)
}

fn st(registers: &mut Registers, memory: &mut Memory) -> Result<(), RizeError> {
    // TODO: Use section-aware reading if MAR/MDR can be partial?
    let address = read_register_u16(registers, "mar")?;
    let data = read_register_u16(registers, "mdr")?;
    memory.write(address, data)
}

fn ld(registers: &mut Registers, memory: &Memory) -> Result<(), RizeError> {
    // TODO: Use section-aware reading if MAR can be partial?
    let address = read_register_u16(registers, "mar")?;
    let data = memory.read(address)?;
    // TODO: Use section-aware writing if MDR can be partial?
    write_register_u16(registers, "mdr", data)
}

fn and(
    arg1: &ArgType,
    arg2: &ArgType,
    arg3_opt: &Option<ArgType>,
    registers: &mut Registers,
) -> Result<(), RizeError> {
    let v1 = get_operand_value(registers, &Memory::new(), arg1)?;
    let v2 = get_operand_value(registers, &Memory::new(), arg2)?;

    if !matches!(arg1, ArgType::Register(_))
        || !matches!(arg2, ArgType::Register(_))
    {
        return Err(RizeError {
            type_: RizeErrorType::Execute,
            message: "AND requires register operands (arg1, arg2).".to_string(),
        });
    }

    let result = v1 & v2;

    let (_dest_register, dest_name) =
        determine_destination_register_mut(registers, arg1, arg3_opt)?;
    // Use section-aware write helper
    write_register_u16(registers, &dest_name, result)
}

fn or(
    arg1: &ArgType,
    arg2: &ArgType,
    arg3_opt: &Option<ArgType>,
    registers: &mut Registers,
) -> Result<(), RizeError> {
    let v1 = get_operand_value(registers, &Memory::new(), arg1)?;
    let v2 = get_operand_value(registers, &Memory::new(), arg2)?;

    if !matches!(arg1, ArgType::Register(_))
        || !matches!(arg2, ArgType::Register(_))
    {
        return Err(RizeError {
            type_: RizeErrorType::Execute,
            message: "OR requires register operands (arg1, arg2).".to_string(),
        });
    }

    let result = v1 | v2;

    let (_dest_register, dest_name) =
        determine_destination_register_mut(registers, arg1, arg3_opt)?;
    // Use section-aware write helper
    write_register_u16(registers, &dest_name, result)
}

fn xor(
    arg1: &ArgType,
    arg2: &ArgType,
    arg3_opt: &Option<ArgType>,
    registers: &mut Registers,
) -> Result<(), RizeError> {
    let v1 = get_operand_value(registers, &Memory::new(), arg1)?;
    let v2 = get_operand_value(registers, &Memory::new(), arg2)?;

    if !matches!(arg1, ArgType::Register(_))
        || !matches!(arg2, ArgType::Register(_))
    {
        return Err(RizeError {
            type_: RizeErrorType::Execute,
            message: "XOR requires register operands (arg1, arg2).".to_string(),
        });
    }

    let result = v1 ^ v2;

    let (_dest_register, dest_name) =
        determine_destination_register_mut(registers, arg1, arg3_opt)?;
    // Use section-aware write helper
    write_register_u16(registers, &dest_name, result)
}

fn not(arg1: &ArgType, registers: &mut Registers) -> Result<(), RizeError> {
    if let ArgType::Register(reg_name) = arg1 {
        // TODO: Use section-aware reading
        let v1 = read_register_u16(registers, reg_name)?;
        let result = !v1;
        // TODO: Use section-aware writing
        write_register_u16(registers, reg_name, result)
    } else {
        Err(RizeError {
            type_: RizeErrorType::Execute,
            message: "NOT requires a Register operand (arg1).".to_string(),
        })
    }
}

fn shl(
    arg1: &ArgType, // Target Register
    arg2: &ArgType, // Amount Immediate (Optional, defaults to 1)
    registers: &mut Registers,
) -> Result<(), RizeError> {
    if let ArgType::Register(target_reg_name) = arg1 {
        let amount = match arg2 {
            ArgType::Immediate(imm) => *imm,
            ArgType::None => 1, // Default shift amount
            _ => {
                return Err(RizeError {
                    type_: RizeErrorType::Execute,
                    message: "SHL amount (arg2) must be Immediate or omitted."
                        .to_string(),
                })
            }
        };

        // TODO: Use section-aware reading
        let value = read_register_u16(registers, target_reg_name)?;
        let result = value << amount;
        // TODO: Use section-aware writing
        write_register_u16(registers, target_reg_name, result)
    } else {
        Err(RizeError {
            type_: RizeErrorType::Execute,
            message: "SHL target (arg1) must be a Register.".to_string(),
        })
    }
}

fn shr(
    arg1: &ArgType, // Target Register
    arg2: &ArgType, // Amount Immediate (Optional, defaults to 1)
    registers: &mut Registers,
) -> Result<(), RizeError> {
    if let ArgType::Register(target_reg_name) = arg1 {
        let amount = match arg2 {
            ArgType::Immediate(imm) => *imm,
            ArgType::None => 1,
            _ => {
                return Err(RizeError {
                    type_: RizeErrorType::Execute,
                    message: "SHR amount (arg2) must be Immediate or omitted."
                        .to_string(),
                })
            }
        };

        // TODO: Use section-aware reading
        let value = read_register_u16(registers, target_reg_name)?;
        let result = value >> amount;
        // TODO: Use section-aware writing
        write_register_u16(registers, target_reg_name, result)
    } else {
        Err(RizeError {
            type_: RizeErrorType::Execute,
            message: "SHR target (arg1) must be a Register.".to_string(),
        })
    }
}

fn wdm(
    arg1: &ArgType,
    arg2: &ArgType,
    arg3: &ArgType,
    mut r_display_memory: ResMut<DisplayMemory>,
    registers: &mut Registers,
    memory: &Memory, // Added memory
) -> Result<(), RizeError> {
    // TODO: Use section-aware reading for register args
    let val1 = get_operand_value(registers, memory, arg1)?;
    let val2 = get_operand_value(registers, memory, arg2)?;
    let val3 = get_operand_value(registers, memory, arg3)?;

    let red = (val1 >> 8) as u8;
    let green = (val1 & 0xFF) as u8;

    let blue = (val2 >> 8) as u8;
    let alpha = (val2 & 0xFF) as u8;

    let x = (val3 >> 8) as u8;
    let y = (val3 & 0xFF) as u8;

    let color = [red, green, blue, alpha];

    r_display_memory.set_pixel(x, y, color)
}
