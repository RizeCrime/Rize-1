use std::fs;
use std::path::PathBuf;

use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;

use super::*;
use crate::*;

#[derive(Resource, Default, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct AzmPrograms(pub Vec<(PathBuf, String)>);

#[derive(Resource)]
pub struct FileCheckTimer(Timer);

pub struct RizeOneInterpreter;

impl Plugin for RizeOneInterpreter {
    fn build(&self, app: &mut App) {
        app.insert_resource(AzmPrograms::default());
        app.insert_resource(FileCheckTimer(Timer::from_seconds(
            5.0,
            TimerMode::Repeating,
        )));

        app.register_type::<AzmPrograms>();

        app.add_systems(Update, check_azm_programs);
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

    match fs::read_dir(azzembly_dir) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        if path.is_file() {
                            if let Some(ext) = path.extension() {
                                if ext == "azm"
                                    && !r_programs
                                        .0
                                        .iter()
                                        .any(|(p, _)| p == &path)
                                {
                                    info!("Found new .azm program: {:?}", path);
                                    let file_stem = path
                                        .file_stem()
                                        .unwrap_or_default()
                                        .to_string_lossy()
                                        .to_string();
                                    r_programs
                                        .0
                                        .push((path.clone(), file_stem));
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
