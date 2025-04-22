use std::sync::Arc;

use bevy::prelude::*;

use super::*;
use crate::*;

#[derive(Component, Reflect, Default)]

pub struct UiElement;

#[derive(Component, Reflect, Default)]
pub struct UiRoot;

impl UiRoot {
    pub fn init() -> impl Bundle {
        (
            UiRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..Default::default()
            },
        )
    }
}

#[derive(Component, Reflect, Default)]
pub struct UiRegister;

#[derive(Component, Reflect, Default)]
pub struct UiText;

#[derive(Component, Reflect, Default)]
pub struct UiConversion;

#[derive(Component, Reflect, Default)]
pub struct UiBit {
    pub value: i8,
    pub src: Arc<str>,
    pub idx: usize,
}

impl UiBit {
    pub fn new(value: i8, src: impl AsRef<str>, idx: usize) -> impl Bundle {
        (
            UiBit {
                value,
                src: Arc::from(src.as_ref()),
                idx,
            },
            Text::new(format!("{value}")),
        )
    }
}

// ---------------- //
// Helper Functions //
// ---------------- //
