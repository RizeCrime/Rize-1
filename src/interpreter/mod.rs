use std::collections::HashMap;
use std::fs::{self, File};
use std::io::BufRead;
use std::path::PathBuf;
use std::str::{FromStr, Lines};

use bevy::prelude::*;
use bevy::tasks::futures_lite::stream::Pending;
use bevy::utils::info;
use bevy_inspector_egui::prelude::*;

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

        // add systems OnEnter, for manual step-through
        app.add_systems(OnEnter(CpuCycleStage::Fetch), fetch);
        app.add_systems(OnEnter(CpuCycleStage::Decode), decode);
        app.add_systems(OnEnter(CpuCycleStage::Execute), execute);

        // add systems as Update, for auto-stepping
        app.add_systems(
            Update,
            (fetch, decode, execute)
                .chain()
                .run_if(in_state(CpuCycleStage::AutoStep)),
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
    // load the current program counter value into lines to operate on,
    // in case the user overwrote the pc register manually.
    // not the most elegant solution; but hey it should work?
    let pc: &mut Register = r_registers.get(PROGRAM_COUNTER).unwrap();
    program.line = pc.read_u16().unwrap() as usize;

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
            r_registers
                .get(PROGRAM_COUNTER)
                .unwrap()
                .store_immediate(program.line as usize)
                .unwrap();
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
                &r_memory,
            )
        }
        OpCode::MUL => {
            let arg3_option = if program.arg3.raw.is_empty() {
                None
            } else {
                Some(program.arg3.parsed.clone())
            };
            mul(
                &program.arg1.parsed,
                &program.arg2.parsed,
                &arg3_option,
                registers,
                &r_memory,
            )
        }
        OpCode::DIV => {
            let arg3_option = if program.arg3.raw.is_empty() {
                None
            } else {
                Some(program.arg3.parsed.clone())
            };
            div(
                &program.arg1.parsed,
                &program.arg2.parsed,
                &arg3_option,
                registers,
                &r_memory,
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
                match get_register_mut(registers, PROGRAM_COUNTER) {
                    Ok(pc_reg) => pc_reg.write_section_u16(target_line as u16),
                    Err(e) => Err(e),
                }
            }
        }
        OpCode::JIZ => {
            // Read flag, handling Result
            match get_operand_value(
                registers,
                &Memory::new(),
                &ArgType::Register(FLAG_ZERO.to_string()),
            ) {
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
                            match get_register_mut(registers, PROGRAM_COUNTER) {
                                Ok(pc_reg) => {
                                    pc_reg.write_section_u16(target_line as u16)
                                }
                                Err(e) => Err(e),
                            }
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
            match get_operand_value(
                registers,
                &Memory::new(),
                &ArgType::Register(FLAG_NEGATIVE.to_string()),
            ) {
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
                            match get_register_mut(registers, PROGRAM_COUNTER) {
                                Ok(pc_reg) => {
                                    pc_reg.write_section_u16(target_line as u16)
                                }
                                Err(e) => Err(e),
                            }
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

/// Determines the value of an operand (Register, Immediate, or Memory Address).
/// Reads section-aware for registers.
fn get_operand_value(
    registers: &mut Registers,
    memory: &Memory,
    arg: &ArgType,
) -> Result<u16, RizeError> {
    match arg {
        ArgType::Register(reg_name) => {
            let register = get_register_mut(registers, reg_name)?;
            // Use the new trait method
            register.read_section_u16()
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
        // If arg3 is None or a comment, use arg1 (which must be a register)
        None | Some(ArgType::None) => {
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
/// 0) if starts with '#'       -> Comment (Ignore)
/// 1) if only characters       -> Register
/// 2) if starts with '0x'      -> MemAddr
/// 3) if is entirely digits    -> Immediate
/// 4) if starts with '.'       -> Symbol
fn parse_arg(arg: &str) -> ArgType {
    if arg.is_empty() {
        return ArgType::None;
    }

    // Rule 0: Comments
    if arg.starts_with('#') {
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
            let register = get_register_mut(registers, dest_reg_name)?;
            // Use the new trait method
            register.write_section_u16(source_value)
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
    // Validate arg1 is a register and get its value
    let v1 = get_operand_value(registers, &Memory::new(), arg1)?;
    let v2 = get_operand_value(registers, &Memory::new(), arg2)?;

    // Ensure arg1 is a register (destination or source)
    if !matches!(arg1, ArgType::Register(_)) {
        return Err(RizeError {
            type_: RizeErrorType::Execute,
            message: "ADD requires the first argument (arg1) to be a register."
                .to_string(),
        });
    }

    // Perform addition using wrapping arithmetic
    let result = v1.wrapping_add(v2);

    // Determine destination register using helper
    let (_dest_register, dest_name) =
        determine_destination_register_mut(registers, arg1, arg3_opt)?;

    // --- Set Flags ---
    // Zero Flag (fz): Set if result is 0
    registers
        .get(FLAG_ZERO)
        .ok_or_else(|| RizeError {
            type_: RizeErrorType::RegisterRead,
            message: format!("Flag register '{}' not found", FLAG_ZERO),
        })?
        .write_bool(result == 0)?;
    // Negative Flag (fn): Set if MSB of result is 1
    registers
        .get(FLAG_NEGATIVE)
        .ok_or_else(|| RizeError {
            type_: RizeErrorType::RegisterRead,
            message: format!("Flag register '{}' not found", FLAG_NEGATIVE),
        })?
        .write_bool(result & 0x8000 != 0)?; // Check MSB
                                            // Carry Flag (fc): Set if unsigned addition resulted in carry
    let carry = (v1 as u32 + v2 as u32) > 0xFFFF;
    registers
        .get(FLAG_CARRY)
        .ok_or_else(|| RizeError {
            type_: RizeErrorType::RegisterRead,
            message: format!("Flag register '{}' not found", FLAG_CARRY),
        })?
        .write_bool(carry)?;
    // Overflow Flag (fo): Set if signed addition resulted in overflow
    let v1_sign = (v1 >> 15) & 1;
    let v2_sign = (v2 >> 15) & 1;
    let result_sign = (result >> 15) & 1;
    let overflow = (v1_sign == v2_sign) && (result_sign != v1_sign);
    registers
        .get(FLAG_OVERFLOW)
        .ok_or_else(|| RizeError {
            type_: RizeErrorType::RegisterRead,
            message: format!("Flag register '{}' not found", FLAG_OVERFLOW),
        })?
        .write_bool(overflow)?;
    // --- End Set Flags ---

    // Get register ref again (determine_... returns name now)
    let dest_register = get_register_mut(registers, &dest_name)?;
    // Use section-aware trait method
    dest_register.write_section_u16(result)
}

fn sub(
    arg1: &ArgType,
    arg2: &ArgType,
    arg3_opt: &Option<ArgType>,
    registers: &mut Registers,
    r_memory: &Memory,
) -> Result<(), RizeError> {
    // Validate arg1 is a register and get its value
    let v1 = get_operand_value(registers, r_memory, arg1)?;
    // Get the value of arg2 (can be Register or Immediate)
    let v2 = get_operand_value(registers, r_memory, arg2)?;

    // Ensure arg1 is a register (destination or source)
    if !matches!(arg1, ArgType::Register(_)) {
        return Err(RizeError {
            type_: RizeErrorType::Execute,
            message: "SUB requires the first argument (arg1) to be a register."
                .to_string(),
        });
    }

    // Perform subtraction using wrapping arithmetic
    let result = v1.wrapping_sub(v2);

    // Determine destination register using helper
    let (_dest_register, dest_name) =
        determine_destination_register_mut(registers, arg1, arg3_opt)?;

    // --- Set Flags ---
    // Zero Flag (fz): Set if result is 0
    registers
        .get(FLAG_ZERO)
        .ok_or_else(|| RizeError {
            type_: RizeErrorType::RegisterRead,
            message: format!("Flag register '{}' not found", FLAG_ZERO),
        })?
        .write_bool(result == 0)?;
    // Negative Flag (fn): Set if MSB of result is 1
    registers
        .get(FLAG_NEGATIVE)
        .ok_or_else(|| RizeError {
            type_: RizeErrorType::RegisterRead,
            message: format!("Flag register '{}' not found", FLAG_NEGATIVE),
        })?
        .write_bool(result & 0x8000 != 0)?; // Check MSB
                                            // Carry Flag (fc): Set if unsigned subtraction resulted in borrow (v1 < v2)
    let borrow = v1 < v2;
    registers
        .get(FLAG_CARRY) // Often called Borrow flag in subtraction context
        .ok_or_else(|| RizeError {
            type_: RizeErrorType::RegisterRead,
            message: format!("Flag register '{}' not found", FLAG_CARRY),
        })?
        .write_bool(borrow)?;
    // Overflow Flag (fo): Set if signed subtraction resulted in overflow
    let v1_sign = (v1 >> 15) & 1;
    let v2_sign = (v2 >> 15) & 1;
    let result_sign = (result >> 15) & 1;
    let overflow = (v1_sign != v2_sign) && (result_sign != v1_sign);
    registers
        .get(FLAG_OVERFLOW)
        .ok_or_else(|| RizeError {
            type_: RizeErrorType::RegisterRead,
            message: format!("Flag register '{}' not found", FLAG_OVERFLOW),
        })?
        .write_bool(overflow)?;
    // --- End Set Flags ---

    // Get register ref again
    let dest_register = get_register_mut(registers, &dest_name)?;
    // Use section-aware trait method
    dest_register.write_section_u16(result)
}

fn st(registers: &mut Registers, memory: &mut Memory) -> Result<(), RizeError> {
    let address_reg = get_register_mut(registers, "mar")?;
    let address = address_reg.read_section_u16()?;
    let data_reg = get_register_mut(registers, "mdr")?;
    let data = data_reg.read_section_u16()?;
    memory.write(address, data)
}

fn ld(registers: &mut Registers, memory: &Memory) -> Result<(), RizeError> {
    let address_reg = get_register_mut(registers, "mar")?;
    let address = address_reg.read_section_u16()?;
    let data = memory.read(address)?;
    let data_reg = get_register_mut(registers, "mdr")?;
    data_reg.write_section_u16(data)
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
    // Get register ref again
    let dest_register = get_register_mut(registers, &dest_name)?;
    // Use section-aware trait method
    dest_register.write_section_u16(result)
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
    // Get register ref again
    let dest_register = get_register_mut(registers, &dest_name)?;
    // Use section-aware trait method
    dest_register.write_section_u16(result)
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
    // Get register ref again
    let dest_register = get_register_mut(registers, &dest_name)?;
    // Use section-aware trait method
    dest_register.write_section_u16(result)
}

fn not(arg1: &ArgType, registers: &mut Registers) -> Result<(), RizeError> {
    if let ArgType::Register(reg_name) = arg1 {
        let register = get_register_mut(registers, reg_name)?;
        let v1 = register.read_section_u16()?;
        let result = !v1;
        register.write_section_u16(result)
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

        let target_register = get_register_mut(registers, target_reg_name)?;
        let value = target_register.read_section_u16()?;
        let result = value << amount;
        target_register.write_section_u16(result)
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

        let target_register = get_register_mut(registers, target_reg_name)?;
        let value = target_register.read_section_u16()?;
        let result = value >> amount;
        target_register.write_section_u16(result)
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
    let val1: u16 = get_operand_value(registers, memory, arg1)?;
    let val2: u16 = get_operand_value(registers, memory, arg2)?;
    let val3: u16 = get_operand_value(registers, memory, arg3)?;

    // info!("Val3: {:#016b}", val3);

    let red: u8 = (val1 >> 8) as u8;
    let green: u8 = (val1 & 0xFF) as u8;

    let blue: u8 = (val2 >> 8) as u8;
    let alpha: u8 = (val2 & 0xFF) as u8;

    let x: u8 = (val3 >> 8) as u8;
    let y: u8 = (val3 & 0xFF) as u8;

    // info(format!("x: {x}, y: {y}"));

    let color = [red, green, blue, alpha];

    r_display_memory.set_pixel(x, y, color)
}

fn mul(
    arg1: &ArgType,
    arg2: &ArgType,
    arg3_opt: &Option<ArgType>,
    registers: &mut Registers,
    r_memory: &Memory,
) -> Result<(), RizeError> {
    // Validate arg1 is a register and get its value
    let v1 = get_operand_value(registers, r_memory, arg1)?;
    // Get the value of arg2 (can be Register or Immediate)
    let v2 = get_operand_value(registers, r_memory, arg2)?;

    // Ensure arg1 is a register (destination or source)
    if !matches!(arg1, ArgType::Register(_)) {
        return Err(RizeError {
            type_: RizeErrorType::Execute,
            message: "MUL requires the first argument (arg1) to be a register."
                .to_string(),
        });
    }

    // Perform multiplication using wrapping arithmetic
    let result = v1.wrapping_mul(v2);

    // Determine destination register using helper
    let (_dest_register, dest_name) =
        determine_destination_register_mut(registers, arg1, arg3_opt)?;

    // --- Set Flags ---
    // Zero Flag (fz): Set if result is 0
    registers
        .get(FLAG_ZERO)
        .ok_or_else(|| RizeError {
            type_: RizeErrorType::RegisterRead,
            message: format!("Flag register '{}' not found", FLAG_ZERO),
        })?
        .write_bool(result == 0)?;
    // Negative Flag (fn): Set if MSB of result is 1
    registers
        .get(FLAG_NEGATIVE)
        .ok_or_else(|| RizeError {
            type_: RizeErrorType::RegisterRead,
            message: format!("Flag register '{}' not found", FLAG_NEGATIVE),
        })?
        .write_bool(result & 0x8000 != 0)?; // Check MSB
                                            // Carry Flag (fc): Set if the product exceeds 16 bits
    let carry = (v1 as u32 * v2 as u32) > 0xFFFF;
    registers
        .get(FLAG_CARRY)
        .ok_or_else(|| RizeError {
            type_: RizeErrorType::RegisterRead,
            message: format!("Flag register '{}' not found", FLAG_CARRY),
        })?
        .write_bool(carry)?;
    // Overflow Flag (fo): Set if signed multiplication resulted in overflow
    // Overflow occurs when the result cannot be correctly represented in 16 bits
    let v1_signed = v1 as i16;
    let v2_signed = v2 as i16;
    let result_i32 = v1_signed as i32 * v2_signed as i32;
    let overflow = result_i32 < i16::MIN as i32 || result_i32 > i16::MAX as i32;
    registers
        .get(FLAG_OVERFLOW)
        .ok_or_else(|| RizeError {
            type_: RizeErrorType::RegisterRead,
            message: format!("Flag register '{}' not found", FLAG_OVERFLOW),
        })?
        .write_bool(overflow)?;
    // --- End Set Flags ---

    // Get register ref again
    let dest_register = get_register_mut(registers, &dest_name)?;
    // Use section-aware trait method
    dest_register.write_section_u16(result)
}

fn div(
    arg1: &ArgType,
    arg2: &ArgType,
    arg3_opt: &Option<ArgType>,
    registers: &mut Registers,
    r_memory: &Memory,
) -> Result<(), RizeError> {
    // Validate arg1 is a register and get its value
    let v1 = get_operand_value(registers, r_memory, arg1)?;
    // Get the value of arg2 (can be Register or Immediate)
    let v2 = get_operand_value(registers, r_memory, arg2)?;

    // Ensure arg1 is a register (destination or source)
    if !matches!(arg1, ArgType::Register(_)) {
        return Err(RizeError {
            type_: RizeErrorType::Execute,
            message: "DIV requires the first argument (arg1) to be a register."
                .to_string(),
        });
    }

    // Check for division by zero
    if v2 == 0 {
        return Err(RizeError {
            type_: RizeErrorType::Execute,
            message: "Division by zero".to_string(),
        });
    }

    // Perform division
    let result = v1.wrapping_div(v2);

    // Determine destination register using helper
    let (_dest_register, dest_name) =
        determine_destination_register_mut(registers, arg1, arg3_opt)?;

    // --- Set Flags ---
    // Zero Flag (fz): Set if result is 0
    registers
        .get(FLAG_ZERO)
        .ok_or_else(|| RizeError {
            type_: RizeErrorType::RegisterRead,
            message: format!("Flag register '{}' not found", FLAG_ZERO),
        })?
        .write_bool(result == 0)?;
    // Negative Flag (fn): Set if MSB of result is 1
    registers
        .get(FLAG_NEGATIVE)
        .ok_or_else(|| RizeError {
            type_: RizeErrorType::RegisterRead,
            message: format!("Flag register '{}' not found", FLAG_NEGATIVE),
        })?
        .write_bool(result & 0x8000 != 0)?; // Check MSB
                                            // Carry Flag (fc): Set if there is a remainder
    let remainder = v1 % v2;
    registers
        .get(FLAG_CARRY)
        .ok_or_else(|| RizeError {
            type_: RizeErrorType::RegisterRead,
            message: format!("Flag register '{}' not found", FLAG_CARRY),
        })?
        .write_bool(remainder != 0)?;
    // Overflow Flag (fo): Set if signed division resulted in overflow
    // Overflow can only occur in signed division when dividing INT_MIN by -1
    let v1_signed = v1 as i16;
    let v2_signed = v2 as i16;
    let overflow = v1_signed == i16::MIN && v2_signed == -1;
    registers
        .get(FLAG_OVERFLOW)
        .ok_or_else(|| RizeError {
            type_: RizeErrorType::RegisterRead,
            message: format!("Flag register '{}' not found", FLAG_OVERFLOW),
        })?
        .write_bool(overflow)?;
    // --- End Set Flags ---

    // Get register ref again
    let dest_register = get_register_mut(registers, &dest_name)?;
    // Use section-aware trait method
    dest_register.write_section_u16(result)
}
