use bevy::prelude::*;

use crate::*;

mod systems;
use systems::*;

pub struct RizeOneUi;

impl Plugin for RizeOneUi {
    fn build(&self, app: &mut App) {

        app.add_systems(Startup, setup_ui);

    }
}

