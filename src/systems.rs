use bevy::prelude::*;

use crate::{constants::AZZEMBLY_DIR, types::{ActiveProgram, AzmPrograms, FileCheckTimer, ProgramSettings}, CpuCycleStage};

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub fn init_cpu(app: &mut App) {
    app.insert_resource(FileCheckTimer(Timer::from_seconds(
        1.0,
        TimerMode::Repeating,
    )));
}

pub fn tick_cpu(
    s_current_stage: Res<State<CpuCycleStage>>,
    mut s_next_stage: ResMut<NextState<CpuCycleStage>>,
    mut r_program: ResMut<ProgramSettings>,
) {
    if !r_program.autostep {
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
            r_program.autostep = false;
        }
        _ => {}
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

    let entries = match std::fs::read_dir(azzembly_dir) {
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