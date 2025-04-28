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

mod types; 
use types::*;

#[derive(Resource, Default, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct AzmPrograms(pub Vec<(PathBuf, String)>);

#[derive(Resource, Default, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct ActiveProgram {
    pub contents: String,
    pub symbols: HashMap<String, usize>,
    pub raw: String,
    pub opcode: OpCode,
    pub arg1: ArgType,
    pub arg2: ArgType,
    pub arg3: ArgType,
}

#[derive(Resource, Default, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct ProgramSettings {
    pub autostep: bool,
    pub autostep_lines: usize,
}

#[derive(Resource, Default, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct ProgramArg {
    pub parsed: ArgType,
}

#[derive(
    Resource, Default, Reflect, InspectorOptions, Clone, Eq, PartialEq, Debug,
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

impl ArgType {
    pub fn store_immediate(
        &self,
        mut registers: Option<&mut Registers>,
        mut memory: Option<&mut Memory>,
        address: Option<u16>,
        val: usize,
    ) -> Result<(), RizeError> {
        match self {
            ArgType::Register(reg_name) => {
                let register = registers.unwrap().get(&reg_name).unwrap();
                register.store_immediate(val as usize)
            }
            ArgType::MemAddr(mem_addr) => {
                let mem = memory.unwrap();
                mem.write(address.unwrap() as u16, val as u16)
            }
            _ => {
                unreachable!()
            }
        }
    }
    /// ### Returns
    /// Result<(result: usize, current: usize), RizeError>  
    /// where 'current' is the Value of Arg1, before the operation.  
    pub fn add(
        &self, 
        mut registers: Option<&mut Registers>, 
        mut memory: Option<&mut Memory>,
        val: usize,
        target: Option<&ArgType>,
    ) -> Result<(usize, usize), RizeError> {

        match self {
            ArgType::Register(reg_name) => {

                let target: &mut Register = match target {
                    Some(ArgType::Register(reg_name)) => {
                        registers.unwrap().get(&reg_name).unwrap()
                    }
                    _ => {
                        registers.unwrap().get(reg_name).unwrap()
                    }
                };
                let current: usize = target.read_u16().unwrap() as usize;
                
                let result: usize = current.wrapping_add(val);
                target.store_immediate(result)?;
                
                return Ok((result, current));
            }
            ArgType::MemAddr(mem_addr) => {
                error!("ADD target (arg1) must be a Register.");
            }
            _ => {
                unreachable!()
            }
        }
        unreachable!()
    }
}

#[derive(Resource)]
pub struct FileCheckTimer(Timer);

pub struct RizeOneInterpreter;

impl Plugin for RizeOneInterpreter {
    fn build(&self, app: &mut App) {
        app.insert_resource(AzmPrograms::default());
        app.insert_resource(ActiveProgram {
            ..Default::default()
        });
        app.insert_resource(ProgramSettings::default());
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
            // (fetch, decode, execute)
            // .chain()
            (auto_step).run_if(in_state(CpuCycleStage::AutoStep)),
        );
    }
}

pub fn auto_step(
    mut commands: Commands,
    r_program_settings: Res<ProgramSettings>,
    mut r_active_program: ResMut<ActiveProgram>,
    mut r_registers: ResMut<Registers>,
    mut r_memory: ResMut<Memory>,
    mut r_display_memory: ResMut<DisplayMemory>,
    r_images: Res<Assets<Image>>,
    mut s_cpu_next: ResMut<NextState<CpuCycleStage>>,
) {
    for _ in 0..r_program_settings.autostep_lines {
        unsafe {
            fetch(
                std::mem::transmute_copy(&r_active_program),
                std::mem::transmute_copy(&s_cpu_next),
                std::mem::transmute_copy(&r_registers),
            );
            decode(std::mem::transmute_copy(&r_active_program));
            execute(
                std::mem::transmute_copy(&r_active_program),
                std::mem::transmute_copy(&r_registers),
                std::mem::transmute_copy(&r_memory),
                std::mem::transmute_copy(&s_cpu_next),
                std::mem::transmute_copy(&r_display_memory),
                std::mem::transmute_copy(&r_images),
            );
        }
    }
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
    mut r_program_settings: ResMut<ProgramSettings>,
) {
    if !r_program_settings.autostep {
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
            r_program_settings.autostep = false;
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
    let mut pc: &mut Register = r_registers.get(PROGRAM_COUNTER).unwrap();
    let pc_val: u16 = pc.read_u16().unwrap();

    // Create an iterator starting from the current line
    let mut lines_iter = program
        .contents
        .lines()
        .skip(pc.read_u16().unwrap() as usize);

    loop {
        if let Some(line_str) = lines_iter.next() {
            let trimmed_line = line_str.trim();

            // Check if the line is empty or a comment
            if trimmed_line.is_empty()
                || trimmed_line.starts_with('#')
                || trimmed_line.starts_with('.')
            {
                pc.store_immediate((pc_val + 1u16) as usize).unwrap(); // Increment line counter for the skipped line
                continue; // Try the next line
            }

            program.raw = trimmed_line.to_string();
            pc.store_immediate((pc_val + 1u16) as usize);
            break;
        } else {
            info!("End of program reached. Halting CPU.");
            next_cpu_stage.set(CpuCycleStage::Halt);

            program.opcode = OpCode::None;
            program.arg1 = ArgType::default();
            program.arg2 = ArgType::default();
            program.arg3 = ArgType::default();

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

    let sections: Vec<String> =
        program.raw.split_whitespace().map(String::from).collect();

    program.opcode = OpCode::from_str(&sections[0]).unwrap_or(OpCode::None);
    program.arg1 = parse_arg(&sections[1]);
    program.arg2 = parse_arg(&sections[2]);
    program.arg3 = parse_arg(&sections[3]);
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
    let mut pc = registers.get(PROGRAM_COUNTER).unwrap();
    let pc_val = pc.read_u16().unwrap();

    // Pre-extract flag values to avoid multiple mutable borrows
    let zero_flag = registers.get(FLAG_ZERO).unwrap().read_u16().unwrap();
    let negative_flag =
        registers.get(FLAG_NEGATIVE).unwrap().read_u16().unwrap();

    let execution_result = match program.opcode {
        OpCode::MOV => mov(&program.arg1, &program.arg2, registers, memory),
        OpCode::ADD => {
            add(&program.arg1, &program.arg2, &program.arg3, registers)
        }
        OpCode::SUB => sub(
            &program.arg1,
            &program.arg2,
            &program.arg3,
            registers,
            &r_memory,
        ),
        OpCode::MUL => mul(
            &program.arg1,
            &program.arg2,
            &program.arg3,
            registers,
            &r_memory,
        ),
        OpCode::DIV => div(
            &program.arg1,
            &program.arg2,
            &program.arg3,
            registers,
            &r_memory,
        ),
        OpCode::ST => st(registers, memory),
        OpCode::LD => ld(registers, memory),
        OpCode::AND => {
            and(&program.arg1, &program.arg2, &program.arg3, registers)
        }
        OpCode::OR => {
            or(&program.arg1, &program.arg2, &program.arg3, registers)
        }
        OpCode::XOR => {
            xor(&program.arg1, &program.arg2, &program.arg3, registers)
        }
        OpCode::NOT => not(&program.arg1, registers),
        OpCode::SHL => shl(&program.arg1, &program.arg2, registers),
        OpCode::SHR => shr(&program.arg1, &program.arg2, registers),
        OpCode::HALT => {
            info!("Halting CPU!");
            next_cpu_stage.set(CpuCycleStage::Halt);
            Ok(())
        }
        OpCode::WDM => wdm(
            &program.arg1,
            &program.arg2,
            &program.arg3,
            r_display_memory,
            registers,
            memory,
        ),
        OpCode::JMP => {
            let target_symbol = if let ArgType::Symbol(sym) = &program.arg1 {
                sym
            } else {
                error!("Operand of 'JMP' must be a symbol.");
                ""
            };
            jmp(&program, target_symbol, pc)
        }
        OpCode::JIZ => {
            let target_symbol = if let ArgType::Symbol(sym) = &program.arg1 {
                sym
            } else {
                error!("Operand of 'JIZ' must be a symbol.");
                ""
            };
            if zero_flag == 0 {
                Ok(())
            } else {
                jmp(&program, target_symbol, pc)
            }
        }
        OpCode::JIN => {
            let target_symbol = if let ArgType::Symbol(sym) = &program.arg1 {
                sym
            } else {
                error!("Operand of 'JIN' must be a symbol.");
                ""
            };
            if negative_flag == 0 {
                Ok(())
            } else {
                jmp(program, target_symbol, pc)
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
            "Execution Error ({:?}): {} (Op: {:?}, Args: '{:?}', '{:?}', '{:?}')",
            e.type_,
            e.message,
            program.opcode,
            program.arg1,
            program.arg2,
            program.arg3
        );
        // Potentially set a CPU Halted state here in the future
    }
}

/// ---------------- ///
/// Helper Functions ///
/// ---------------- ///


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

fn increment_pc(registers: &mut Registers) {}

fn write_pc(registers: &mut Registers, value: usize) {}

fn jmp(
    program: &ActiveProgram,
    target: &str,
    pc: &mut Register,
) -> Result<(), RizeError> {
    let target_line = program.symbols.get(target).copied().unwrap_or_default();
    pc.store_immediate(target_line as usize)
}

fn mov(
    arg1: &ArgType, // Destination (Register or MemAddr)
    arg2: &ArgType, // Source (Register, Immediate, MemAddr)
    registers: &mut Registers,
    memory: &mut Memory, // Needs mutable memory for MemAddr dest
) -> Result<(), RizeError> {
    if !(matches!(arg1, ArgType::Register(_))
        || matches!(arg1, ArgType::MemAddr(_)))
    {
        return Err(RizeError {
            type_: RizeErrorType::Execute,
            message: "MOV destination (arg1) must be Register.".to_string(),
        });
    }

    match arg2 {
        ArgType::Immediate(val) => arg1.store_immediate(
            Some(registers),
            Some(memory),
            None,
            *val as usize,
        ),
        ArgType::Register(reg_name) => {
            let val: u16 = registers.get(reg_name).unwrap().read_u16()?;
            arg1.store_immediate(
                Some(registers),
                Some(memory),
                None,
                val as usize,
            )
        }
        ArgType::MemAddr(addr) => {
            let val: u16 = memory.read(*addr)?;
            arg1.store_immediate(
                Some(registers),
                Some(memory),
                Some(*addr),
                val as usize,
            )
        }
        _ => {
            return Err(RizeError {
                type_: RizeErrorType::Execute,
                message:
                    "MOV source (arg2) must be Register, Immediate, or MemAddr."
                        .to_string(),
            })
        }
    }
}

fn add(
    arg1: &ArgType,
    arg2: &ArgType,
    arg3: &ArgType,
    registers: &mut Registers,
    memory: &mut Memory,
) -> Result<(), RizeError> {
    if !(matches!(arg1, ArgType::Register(_))
        || matches!(arg1, ArgType::MemAddr(_)))
    {
        return Err(RizeError {
            type_: RizeErrorType::Execute,
            message:
                "ADD destination (arg1) must be Register or Memory Address."
                    .to_string(),
        });
    }

    if !(matches!(arg2, ArgType::Register(_))
        || matches!(arg2, ArgType::Immediate(_))
        || matches!(arg2, ArgType::MemAddr(_)))
    {
        return Err(RizeError { 
            type_: RizeErrorType::Execute, 
            message: "ADD requires the second argument (arg2) to be a Register, Immediate, or Memory Address"
            .to_string() 
        });
    }

    if !(matches!(arg3, ArgType::Register(_)) || !matches!(arg3, ArgType::None))
    {
        return Err(RizeError {
            type_: RizeErrorType::Execute,
            message: "ADD requires the third argument (arg3) to be a Register."
                .to_string(),
        });
    }

    let val: usize = match arg2 {
        ArgType::Immediate(val) => *val as usize,
        ArgType::Register(reg_name) => {
            registers.get(reg_name).unwrap().read_u16()? as usize
        }
        ArgType::MemAddr(addr) => memory.read(*addr).unwrap() as usize,
        _ => {
            return Err(RizeError {
                type_: RizeErrorType::Execute,
                message: "ADD source (arg2) must be Register, Immediate, or MemAddr.".to_string()
            })
    }};

    let (result, v1) = match arg3 {
        ArgType::Register(reg_name) => {
            arg3.add(Some(registers), Some(memory), val, Some(arg1))
        }
        _ => {
            arg1.add(Some(registers), Some(memory), val, None)
        }
    }?;

    // --- Set Flags ---
    // Zero Flag (fz): Set if result is 0
    registers
        .get(FLAG_ZERO).unwrap()
        .write_bool(result == 0)?;
    // Negative Flag (fn): Set if MSB of result is 1
    registers
        .get(FLAG_NEGATIVE).unwrap()
        .write_bool(result & 0x8000 != 0)?; // Check MSB
                                            // Carry Flag (fc): Set if unsigned addition resulted in carry
    let carry = (v1 as u32 + val as u32) > 0xFFFF;
    registers
        .get(FLAG_CARRY)
        .ok_or_else(|| RizeError {
            type_: RizeErrorType::RegisterRead,
            message: format!("Flag register '{}' not found", FLAG_CARRY),
        })?
        .write_bool(carry)?;
    // Overflow Flag (fo): Set if signed addition resulted in overflow
    let v1_sign = (v1 >> 15) & 1;
    let v2_sign = (val >> 15) & 1;
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

    Ok(())
}

fn sub(
    arg1: &ArgType,
    arg2: &ArgType,
    arg3: &ArgType,
    registers: &mut Registers,
    r_memory: &mut Memory,
) -> Result<(), RizeError> {
    let val = match arg2 {
        ArgType::Register(reg_name) => {
            let register = registers.get(reg_name).unwrap();
            register.read_u16().unwrap() as usize
        }
        ArgType::Immediate(value) => *value as usize,
        ArgType::MemAddr(addr) => r_memory.read(*addr)? as usize,
        _ => {
            return Err(RizeError {
                type_: RizeErrorType::Execute,
                message: "SUB source (arg2) must be Register, Immediate, or MemAddr.".to_string()
            })
    }};

    let (result, v1) = match arg3 {
        ArgType::Register(reg_name) => {
            arg3.add(Some(registers), Some(r_memory), !val + 1, Some(arg1))
        }
        _ => {
            arg1.add(Some(registers), Some(r_memory), !val + 1, None)
        }
    }?;

    // --- Set Flags ---
    // Zero Flag (fz): Set if result is 0
    registers
        .get(FLAG_ZERO).unwrap()
        .write_bool(result == 0)?;
    // Negative Flag (fn): Set if MSB of result is 1  
    registers
        .get(FLAG_NEGATIVE).unwrap()
        .write_bool(result & 0x8000 != 0)?;
    // Carry Flag (fc): Set if unsigned subtraction resulted in borrow
    let carry = v1 < val;
    registers
        .get(FLAG_CARRY)
        .ok_or_else(|| RizeError {
            type_: RizeErrorType::RegisterRead,
            message: format!("Flag register '{}' not found", FLAG_CARRY),
        })?
        .write_bool(carry)?;
    // Overflow Flag (fo): Set if signed subtraction resulted in overflow
    let v1_sign = (v1 >> 15) & 1;
    let v2_sign = (val >> 15) & 1;
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

    Ok(())
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
    arg3: &ArgType,
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
    arg3: &ArgType,
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
    arg3: &ArgType,
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

    let arg3_opt = if matches!(arg3, ArgType::None) {
        None
    } else {
        Some(arg3.clone())
    };

    let (_dest_register, dest_name) =
        determine_destination_register_mut(registers, arg1, &arg3_opt)?;
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
    arg3: &ArgType,
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
    arg3: &ArgType,
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
