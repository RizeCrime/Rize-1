use bevy::prelude::*;

use crate::{
    constants::{DISPLAY_HEIGHT, DISPLAY_WIDTH},
    types::{RizeError, RizeErrorType},
    CpuCycleStage,
};

pub struct RizeOneDisplay;

impl Plugin for RizeOneDisplay {
    fn build(&self, app: &mut App) {
        app.insert_resource(DisplayMemory::init());
        app.register_type::<Display>();

        app.add_systems(OnEnter(CpuCycleStage::Startup), reset_display);
    }
}

/// ### Dev Metadata
/// Each Row has 256 Columns,
/// which each have 256 Pixels,
/// which each have u8 Values for RGBA
#[derive(Resource, Reflect)]
pub struct DisplayMemory {
    pixels: [[[u8; 4]; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
}

#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
impl DisplayMemory {
    pub fn init() -> Self {
        let mut pixels = [[[0; 4]; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
        for x in 0..DISPLAY_WIDTH {
            for y in 0..DISPLAY_HEIGHT {
                pixels[x][y] = [(x + 100) as u8, 100u8, (y + 100) as u8, 255u8];
            }
        }

        Self { pixels }
    }

    pub fn set_pixel(
        &mut self,
        x: u8,
        y: u8,
        color: [u8; 4],
    ) -> Result<(), RizeError> {
        if (x as usize) >= DISPLAY_WIDTH {
            return Err(RizeError {
                type_: RizeErrorType::Display(format!(
                    "X coordinate {} out of bounds (width is {})",
                    x, DISPLAY_WIDTH
                )),
            });
        }
        if (y as usize) >= DISPLAY_HEIGHT {
            return Err(RizeError {
                type_: RizeErrorType::Display(format!(
                    "Y coordinate {} out of bounds (height is {})",
                    y, DISPLAY_HEIGHT
                )),
            });
        }

        self.pixels[x as usize][y as usize] = color;
        Ok(())
    }

    pub fn get_pixel(&self, x: u16, y: u16) -> Result<[u8; 4], RizeError> {
        self.pixels
            .get(x as usize)
            .ok_or_else(|| RizeError {
                type_: RizeErrorType::Display(format!(
                    "X coordinate {} out of bounds",
                    x
                )),
            })?
            .get(y as usize)
            .ok_or_else(|| RizeError {
                type_: RizeErrorType::Display(format!(
                    "Y coordinate {} out of bounds",
                    y
                )),
            })
            .copied()
    }
}

pub fn reset_display(mut r_display: ResMut<DisplayMemory>) {
    r_display.pixels = DisplayMemory::init().pixels;
}
