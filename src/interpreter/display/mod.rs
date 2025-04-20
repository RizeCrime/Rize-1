use bevy::prelude::*;

use super::*;
use crate::*;

pub struct RizeOneDisplay;

impl Plugin for RizeOneDisplay {
    fn build(&self, app: &mut App) {
        app.insert_resource(DisplayMemory::init());
        app.register_type::<Display>();
    }
}

/// ### Dev Metadata
/// Each Row has 256 Columns,
/// which each have 256 Pixels,
/// which each have u8 Values for RGBA
#[derive(Resource, Reflect)]
pub struct DisplayMemory {
    pixels: [[[u8; 4]; 256]; 256],
}

impl DisplayMemory {
    pub fn init() -> Self {
        let mut pixels = [[[0; 4]; 256]; 256];
        for x in 0..256 {
            for y in 0..256 {
                pixels[x][y] = [x as u8, 0u8, y as u8, 255u8];
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
        self.pixels[x as usize][y as usize] = color;
        Ok(())
    }

    pub fn get_pixel(&self, x: u16, y: u16) -> Result<[u8; 4], RizeError> {
        self.pixels
            .get(x as usize)
            .ok_or_else(|| RizeError {
                type_: RizeErrorType::Display,
                message: format!("X coordinate {} out of bounds", x),
            })?
            .get(y as usize)
            .ok_or_else(|| RizeError {
                type_: RizeErrorType::Display,
                message: format!("Y coordinate {} out of bounds", y),
            })
            .map(|pixel| *pixel)
    }
}
