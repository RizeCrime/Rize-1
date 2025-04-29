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
use systems::setup_camera;
use types::{ActiveProgram, AzmPrograms, ProgramSettings};

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

pub struct RizeOne;

impl Plugin for RizeOne {
    fn build(&self, app: &mut App) {
        app.init_state::<CpuCycleStage>();

        #[cfg(debug_assertions)]
        app.insert_resource(types::Registers::default())
            .insert_resource(types::SystemMemory::default());

        app.insert_resource(DisplayMemory::init())
            .insert_resource(AzmPrograms::default())
            .insert_resource(ActiveProgram::default())
            .insert_resource(ProgramSettings::default());

        app.add_systems(Startup, setup_camera);
        // app.add_systems(OnEnter(CpuCycleStage::Startup), setup_registers);

        app.add_plugins(ui::RizeOneUi);
        // app.add_plugins(interpreter::RizeOneInterpreter);

        // #[cfg(debug_assertions)]
        // app.add_plugins(
        //     // StateInspectorPlugin::<CpuCycleStage>::default(),
        //     // ResourceInspectorPlugin::<types::Registers>::default(),
        //     // ResourceInspectorPlugin::<types::SystemMemory>::default(),
        // );
    }
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
