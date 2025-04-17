use std::fs::{self, File};
use std::io::BufRead;
use std::path::PathBuf;

use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;

use super::*;
use crate::*;

#[derive(Resource, Default, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct AzmPrograms(pub Vec<(PathBuf, String)>);

#[derive(Resource, Default, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct ActiveProgram {
    pub path: PathBuf,
    pub file_stem: String,
    pub contents: String,
    pub line: usize,
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

#[derive(Resource, Default, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub enum ArgType {
    #[default]
    None,
    Error,
    Register(String),
    MemAddr(u16),
    Immediate(i16),
}

#[derive(Resource)]
pub struct FileCheckTimer(Timer);

pub struct RizeOneInterpreter;

impl Plugin for RizeOneInterpreter {
    fn build(&self, app: &mut App) {
        app.insert_resource(AzmPrograms::default());
        app.insert_resource(ActiveProgram::default());
        app.insert_resource(FileCheckTimer(Timer::from_seconds(
            5.0,
            TimerMode::Repeating,
        )));

        app.register_type::<AzmPrograms>();
        app.register_type::<ActiveProgram>();

        #[cfg(debug_assertions)]
        app.add_plugins(ResourceInspectorPlugin::<ActiveProgram>::default());

        app.add_systems(Update, check_azm_programs);

        app.add_systems(OnEnter(CpuCycleStage::Fetch), fetch);
        app.add_systems(OnEnter(CpuCycleStage::Decode), decode);
        app.add_systems(OnEnter(CpuCycleStage::Execute), execute);
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
    debug!("Checking for .azm programs in {}", azzembly_dir);

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

/// -------------- ///
/// Update Systems ///
/// -------------- ///

pub fn fetch(mut r_active_program: ResMut<ActiveProgram>) {
    let mut program = r_active_program.as_mut();

    // Create an iterator starting from the current line
    let mut lines_iter = program.contents.lines().skip(program.line);

    loop {
        if let Some(line_str) = lines_iter.next() {
            let trimmed_line = line_str.trim();

            // Check if the line is empty or a comment
            if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
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

            program.line += 1; // Increment line counter for the processed line
            break; // Instruction fetched, exit loop
        } else {
            // Reached end of file while searching for a non-empty/non-comment line
            program.raw_opcode = String::new(); // Indicate end or issue
            program.arg1 = ProgramArg::default();
            program.arg2 = ProgramArg::default();
            program.arg3 = ProgramArg::default();
            // program.line remains at the position *after* the last line
            break;

            todo!("Set CPU State as Halted.");
        }
    }
}
}
