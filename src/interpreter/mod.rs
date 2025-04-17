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
    #[reflect(ignore)]
    pub buf_reader: Option<std::io::BufReader<std::fs::File>>,
    pub opcode: String,
    pub arg1: String,
    pub arg2: String,
    pub arg3: String,
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
                        }
                    }
                    Err(e) => error!("Error reading directory entry: {}", e),
                }
            }
        }
        Err(e) => error!("Error reading directory {}: {}", azzembly_dir, e),
    }
}
