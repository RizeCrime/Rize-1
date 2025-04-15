#![allow(
    dead_code,
    unused_imports,
    unused_mut,
    unused_variables
)]

use bevy::prelude::*;

use bevy::{
    log::LogPlugin,
    prelude::*,
    window::{PresentMode, WindowResolution},
};
// use bevy_egui::EguiPlugin;
// use bevy_mod_picking::DefaultPickingPlugins;

#[cfg(feature = "inspector")]
use bevy_inspector_egui::quick::{
    ResourceInspectorPlugin, StateInspectorPlugin, WorldInspectorPlugin,
};

mod constants;
pub use constants::*;
mod types;
pub use types::*;
mod systems;
pub use systems::*;

mod components;
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
            })
    );

    #[cfg(feature = "inspector")]
    bevy_app.add_plugins((
        WorldInspectorPlugin::new(),
        ResourceInspectorPlugin::<types::Registers>::default()
    ));

    bevy_app.add_plugins(RizeOne);

    bevy_app.run();
}

pub struct RizeOne;

impl Plugin for RizeOne {
    fn build(&self, app: &mut App) {

        app.insert_resource(types::Registers::new());
        app.add_systems(Startup, (setup_camera, setup_registers));

        app.add_plugins(ui::RizeOneUi);

    }
}