use bevy::prelude::*;

use crate::*;

mod systems;
use systems::*;

mod components;
pub use components::*;

pub struct RizeOneUi;

impl Plugin for RizeOneUi {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                setup_ui_root,
                setup_ui_registers,
                setup_ui_cpu_cycle_stage,
                setup_available_programs,
            )
                .chain(),
        );

        app.add_systems(Update, available_programs);
    }
}
