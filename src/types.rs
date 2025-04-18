use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::{interpreter::ArgType, *};

#[derive(Resource, Reflect, Default)]
pub struct Register {
    #[reflect(ignore)]
    bits: Arc<Mutex<Vec<i8>>>,
}

impl Register {
    pub fn init(length: usize) -> Self {
        Self {
            bits: Arc::new(Mutex::new(vec![0i8; length])),
        }
    }
}

pub trait RegisterTrait {
    fn read(&self) -> Result<Vec<i8>, &'static str>;
    fn read_u16(&self) -> Result<u16, &'static str>;
    fn read_ascii(&self) -> Result<String, &'static str>;
    fn read_hex(&self) -> Result<String, &'static str>;

    #[doc(hidden)]
    fn store_immediate(&self, value: usize) -> Result<(), RizeError>;
    fn store_memaddr() -> Result<(), &'static str>;

    fn store(&self, value: ArgType) -> Result<(), RizeError>;
}

impl RegisterTrait for Register {
    fn read(&self) -> Result<Vec<i8>, &'static str> {
        let bits = self
            .bits
            .lock()
            .map_err(|_| "Failed to acquire lock for reading")?;
        Ok(bits.clone())
    }

    fn read_u16(&self) -> Result<u16, &'static str> {
        let bits = self.read()?;
        let mut value: u16 = 0;
        for (i, bit) in bits.iter().enumerate() {
            value |= (*bit as u16) << (CPU_BITTAGE - 1 - i);
        }
        Ok(value)
    }

    fn read_ascii(&self) -> Result<String, &'static str> {
        let value_u16 = self.read_u16()?;
        let bytes = value_u16.to_le_bytes(); // Get bytes in little-endian order
        let ascii_string: String = bytes
            .iter()
            .map(|&b| if b.is_ascii_graphic() { b as char } else { ' ' }) // Replace non-graphic with space
            .collect();
        Ok(ascii_string)
    }

    fn read_hex(&self) -> Result<String, &'static str> {
        let value_u16 = self.read_u16()?;
        Ok(format!("0x{:04X}", value_u16)) // Format as 4-digit hex with 0x prefix
    }

    /// ### Dev Metadata
    /// 1) truncate usize to u16, using Most Significant Bit First Ordering  
    /// 2) Store the 16 bits into the register  
    ///     - The Register is guaranteed to be [crate::constants::CPU_BITTAGE] long
    ///     - See [crate::systems::setup_registers]
    fn store_immediate(&self, value: usize) -> Result<(), RizeError> {
        let mut bits = self.bits.lock().map_err(|_| RizeError {
            type_: RizeErrorType::Execute,
            message: "Failed to acquire lock for store_immediate".to_string(),
        })?;
        let value_u16 = value as u16;

        for i in 0..CPU_BITTAGE {
            // Calculate the bit index from the right (LSB = 0) in the u16 value
            let bit_idx_from_lsb = CPU_BITTAGE - 1 - i;
            let bit = (value_u16 >> bit_idx_from_lsb) & 1;
            bits[i] = bit as i8;
        }
        Ok(())
    }

    fn store_memaddr() -> Result<(), &'static str> {
        todo!()
    }

    fn store(&self, value: ArgType) -> Result<(), RizeError> {
        match value {
            ArgType::Register(reg_name) => Err(RizeError {
                type_: RizeErrorType::Execute,
                message:
                    "Storing Type 'Register' in Type 'Register' is not allowed!"
                        .to_string(),
            }),
            ArgType::MemAddr(addr) => {
                todo!()
            }
            ArgType::Immediate(imm) => {
                self.store_immediate(imm as u16 as usize)
            }
            ArgType::None => Err(RizeError {
                type_: RizeErrorType::Execute,
                message:
                    "Storing Type 'None' in Type 'Register' is not allowed!"
                        .to_string(),
            }),
            ArgType::Error => {
                todo!()
            }
        }
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

    /// ### Dev Metadata
    /// - Check if `name` starts with 'g'
    /// - If No:
    ///     - Match exact name and return entire register
    /// - If Yes, return only relevant part of register, by checking the third letter:
    ///     - 'a' -> full width
    ///     - 'b' -> lower half of 'a'
    ///     - 'c' -> lower half of 'b'
    ///     - 'd' -> lower half of 'c'
    pub fn get(&self, name: &str) -> Option<&Register> {
        info!("Getting Register by Name... ");
        info!("Searching for: '{name}'");
        info!("Available Registers: {:?}", self.all.keys());

        self.all.get(name.to_ascii_lowercase().as_str())
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

#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Resource,
    Default,
    Reflect,
    InspectorOptions,
)]
#[reflect(Resource, InspectorOptions)]
pub enum OpCode {
    #[default]
    None,
    LD,
    ST,
    SWP,
    MOV,
    ADD,
    SUB,
    NOT,
    AND,
    OR,
    XOR,
    HALT,
    NOP,
    JMP,
    JIZ,
    JIN,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseOpCodeError;

impl std::str::FromStr for OpCode {
    type Err = ParseOpCodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "LD" => Ok(OpCode::LD),
            "ST" => Ok(OpCode::ST),
            "SWP" => Ok(OpCode::SWP),
            "MOV" => Ok(OpCode::MOV),
            "ADD" => Ok(OpCode::ADD),
            "SUB" => Ok(OpCode::SUB),
            "NOT" => Ok(OpCode::NOT),
            "AND" => Ok(OpCode::AND),
            "OR" => Ok(OpCode::OR),
            "XOR" => Ok(OpCode::XOR),
            "HALT" => Ok(OpCode::HALT),
            "NOP" => Ok(OpCode::NOP),
            "JMP" => Ok(OpCode::JMP),
            "JIZ" => Ok(OpCode::JIZ),
            "JIN" => Ok(OpCode::JIN),
            _ => Err(ParseOpCodeError),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RizeErrorType {
    Fetch,
    Decode,
    Execute,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RizeError {
    pub type_: RizeErrorType,
    pub message: String,
}
