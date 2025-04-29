use bevy::prelude::*;
use bevy_simple_text_input::TextInputSystem;
use systems::{
    available_programs, setup_available_programs, setup_control_panel,
    setup_display, setup_instruction_ui, setup_ui_registers, setup_ui_root,
    update_control_panel, update_display, update_instruction_ui,
    update_register_parsed, update_registers,
};
use types::UiBit;

use crate::{
    constants::{DISPLAY_HEIGHT, DISPLAY_WIDTH},
    types::{RizeError, RizeErrorType},
};

mod display;
mod systems;
mod types;

pub struct RizeOneUi;

impl Plugin for RizeOneUi {
    fn build(&self, app: &mut App) {
        // app.register_type::<UiRoot>();
        // app.register_type::<UiElement>();
        // app.register_type::<UiText>();
        // app.register_type::<UiRegister>();
        app.register_type::<UiBit>();

        app.add_systems(
            Startup,
            (
                setup_ui_root,
                (
                    setup_ui_registers,
                    setup_control_panel,
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
                update_registers,
                update_register_parsed,
                // update_control_panel,
                available_programs,
                update_instruction_ui,
                update_display,
                (update_control_panel).after(TextInputSystem),
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
                type_: RizeErrorType::Display(format!(
                    "Coordinates ({}, {}) out of bounds",
                    x, y
                )),
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
