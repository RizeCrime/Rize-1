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
                setup_gp_registers,
                setup_core_registers,
                setup_ui_cpu_cycle_stage,
                setup_available_programs,
                setup_instruction_ui,
            )
                .chain(),
        );

        app.add_systems(
            Update,
            (
                update_cpu_cycle_stage,
                available_programs,
                update_registers,
                update_register_parsed,
                update_instruction_ui,
            ),
        );
    }
}
