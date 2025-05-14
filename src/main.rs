#![allow(clippy::type_complexity, clippy::unnecessary_cast)]

use bevy::{
    log::LogPlugin,
    prelude::*,
    window::{PresentMode, WindowResolution},
};

use bevy_simple_text_input::TextInputPlugin;

#[cfg(debug_assertions)]
#[allow(unused_imports)]
use bevy_inspector_egui::quick::{
    ResourceInspectorPlugin, StateInspectorPlugin, WorldInspectorPlugin,
};
#[cfg(debug_assertions)]
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
#[cfg(debug_assertions)]
use bevy_screen_diagnostics::{
    ScreenDiagnosticsPlugin, ScreenEntityDiagnosticsPlugin,
    ScreenFrameDiagnosticsPlugin,
};
use display::DisplayMemory;
use systems::{check_programs, load_program, setup_camera};
use types::{
    ActiveProgram, AzmPrograms, Byte, FileCheckTimer, ProgramSettings,
    Registers,
};

mod constants;
mod display;
mod interpreter;
mod systems;
pub mod types;
mod ui;

fn main() {
    let mut bevy_app = App::new();

    #[cfg(not(debug_assertions))]
    bevy_app.add_plugins(DefaultPlugins);

    #[cfg(debug_assertions)]
    bevy_app.add_plugins(
        DefaultPlugins
            .set(LogPlugin {
                level: bevy::log::Level::INFO,
                ..Default::default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(1920.0, 1080.0),
                    present_mode: PresentMode::AutoNoVsync,
                    ..Default::default()
                }),
                ..Default::default()
            }),
    );

    #[cfg(debug_assertions)]
    bevy_app.add_plugins((
        DefaultInspectorConfigPlugin,
        WorldInspectorPlugin::new(),
        // ResourceInspectorPlugin::<types::Registers>::default(),
    ));

    #[cfg(debug_assertions)]
    bevy_app.add_plugins((
        ScreenDiagnosticsPlugin::default(),
        ScreenFrameDiagnosticsPlugin,
        ScreenEntityDiagnosticsPlugin,
    ));

    bevy_app.add_plugins(TextInputPlugin);

    bevy_app.add_plugins(RizeOne);

    bevy_app.run();
}

#[derive(Debug, Default, Resource, Reflect)]
pub struct ChunkSize(usize);

pub struct RizeOne;

impl Plugin for RizeOne {
    fn build(&self, app: &mut App) {
        app.init_state::<CpuCycleStage>();

        #[cfg(debug_assertions)]
        app.register_type::<ActiveProgram>()
            .register_type::<Byte>()
            .register_type::<ChunkSize>()
            .add_plugins(ResourceInspectorPlugin::<ActiveProgram>::default())
            .add_plugins(ResourceInspectorPlugin::<ChunkSize>::default());
        // .add_plugins(ResourceInspectorPlugin::<Registers>::default());

        app.insert_resource(types::Registers::default())
            .insert_resource(DisplayMemory::init())
            .insert_resource(types::SystemMemory::default())
            .insert_resource(AzmPrograms::default())
            .insert_resource(ActiveProgram::default())
            .insert_resource(ChunkSize(100_000))
            .insert_resource(ProgramSettings::default())
            .insert_resource(FileCheckTimer(Timer::from_seconds(
                0.25,
                TimerMode::Repeating,
            )));

        app.add_plugins(ui::RizeOneUi)
            .add_plugins(interpreter::RizeOneInterpreterPlugin);

        app.add_systems(Startup, setup_camera)
            .add_systems(Update, (check_programs, load_program, update_debug))
            .add_systems(
                OnEnter(CpuCycleStage::Startup),
                systems::setup_registers,
            );

        // #[cfg(debug_assertions)]
        // app
        //     .add_plugins(StateInspectorPlugin::<CpuCycleStage>::default())
        //     .add_plugins(ResourceInspectorPlugin::<Registers>::default())
        //     .add_plugins(ResourceInspectorPlugin::<SystemMemory>::default());
    }
}

fn update_debug(mut registers: ResMut<Registers>) {
    registers.all.iter_mut().for_each(|register| {
        let byte = &mut register.1.byte;
        byte.size = byte.dsb.lock().unwrap().as_ref().get_size();
    });
}

#[derive(States, Default, Debug, Reflect, Hash, PartialEq, Eq, Clone, Copy)]
pub enum CpuCycleStage {
    #[default]
    Startup,
    Fetch,
    Decode,
    Execute,
    AutoStep,
    Halt,
}
