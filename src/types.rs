use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use std::collections::HashMap;

use crate::*;

#[derive(Resource, Reflect, Default)]

pub struct Register {
    bits: Vec<i8>,
}

impl Register {
    pub fn init(length: usize) -> Self {
        Self {
            bits: vec![0i8; length],
        }
    }
}

pub trait RegisterTrait {
    fn read(&self) -> Vec<i8>;
}

impl RegisterTrait for Register {
    fn read(&self) -> Vec<i8> {
        self.bits.clone()
    }
}

/// # Inner Structure with Labels
/// - HashMap<Name, RegisterInstance>
#[derive(Resource, Reflect, Default, InspectorOptions)]

pub struct Registers {
    all: HashMap<String, Register>,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            all: HashMap::new(),
        }
    }

    pub fn all(&self) -> &HashMap<String, Register> {
        &self.all
    }

    pub fn insert(&mut self, name: String, register: Register) {
        self.all.insert(name, register);
    }
}

pub trait BitToString {
    fn bit_to_string(&self) -> String;
}

impl BitToString for i8 {
    fn bit_to_string(&self) -> String {
        match self {
            0 => "0".to_string(),
            1 => "1".to_string(),
            _ => "?".to_string(),
        }
    }
}

#[derive(
    Default, Debug, Eq, PartialEq, Resource, Reflect, InspectorOptions,
)]
pub struct Memory {
    bytes: Vec<u16>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            bytes: vec![0u16; MEMORY_SIZE_BYTES],
        }
    }
}
