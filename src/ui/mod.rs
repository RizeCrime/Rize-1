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
                (
                    setup_gp_registers,
                    setup_core_registers,
                    setup_ui_cpu_cycle_stage,
                    setup_available_programs,
                    setup_instruction_ui,
                    setup_display,
                ),
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
                update_display,
            ),
        );
    }
}

#[derive(Resource)]
pub struct PixelDisplay {
    h_image: Handle<Image>,
}

impl PixelDisplay {
    pub fn set_pixel(
        &self,
        x: usize,
        y: usize,
        color: [u8; 4],
        mut r_images: &mut ResMut<Assets<Image>>,
    ) -> Result<(), RizeError> {
        if x >= DISPLAY_WIDTH || y >= DISPLAY_HEIGHT {
            return Err(RizeError {
                type_: RizeErrorType::Display,
                message: format!("Coordinates ({}, {}) out of bounds", x, y),
            });
        }

        let image: &mut Image = r_images.get_mut(&self.h_image).unwrap();
        let image_data: &mut [u8] = image.data.as_mut_slice();

        let index = ((y * DISPLAY_WIDTH + x) * 4) as usize;
        if index + 4 <= image_data.len() {
            image_data[index..index + 4].copy_from_slice(&color);
        } else {
            warn!("Pixel at ({x}, {y}) Out Of Bounds!");
        }

        Ok(())
    }
}
