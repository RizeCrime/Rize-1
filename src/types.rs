use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::{interpreter::ArgType, *};

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuHaltedEvent;

#[derive(Resource, Reflect, Default, Debug)]
pub struct Register {
    #[reflect(ignore)]
    bits: Arc<Mutex<Vec<i8>>>,
    pub section: char, // Added section field
}

impl Register {
    pub fn init(length: usize) -> Self {
        Self {
            bits: Arc::new(Mutex::new(vec![0i8; length])),
            section: 'a', // Default to full width
        }
    }
}

pub trait RegisterTrait {
    fn read(&self) -> Result<Vec<i8>, &'static str>;
    fn read_lower_half(&self) -> Result<Vec<i8>, &'static str>;
    fn read_lower_quarter(&self) -> Result<Vec<i8>, &'static str>;
    fn read_lower_eigth(&self) -> Result<Vec<i8>, &'static str>;

    fn read_u16(&self) -> Result<u16, &'static str>;
    fn read_ascii(&self) -> Result<String, &'static str>;
    fn read_hex(&self) -> Result<String, &'static str>;

    #[doc(hidden)]
    fn store_immediate(&self, value: usize) -> Result<(), RizeError>;

    fn write_bool(&self, value: bool) -> Result<(), RizeError>;
    fn write_lower_half(&self, value: Vec<i8>) -> Result<(), RizeError>;
    fn write_lower_quarter(&self, value: Vec<i8>) -> Result<(), RizeError>;
    fn write_lower_eigth(&self, value: Vec<i8>) -> Result<(), RizeError>;

    /// Reads the u16 value from the register, respecting its current section setting.
    fn read_section_u16(&self) -> Result<u16, RizeError>;

    /// Writes a u16 value to the register, respecting its current section setting.
    fn write_section_u16(&self, value: u16) -> Result<(), RizeError>;
}

impl RegisterTrait for Register {
    fn read(&self) -> Result<Vec<i8>, &'static str> {
        let bits = self
            .bits
            .lock()
            .map_err(|_| "Failed to acquire lock for reading")?;
        Ok(bits.clone())
    }
    fn read_lower_half(&self) -> Result<Vec<i8>, &'static str> {
        let bits = self.read()?;
        Ok(bits[CPU_BITTAGE / 2..].to_vec())
    }
    fn read_lower_quarter(&self) -> Result<Vec<i8>, &'static str> {
        let bits = self.read()?;
        Ok(bits[CPU_BITTAGE * 3 / 4..].to_vec())
    }
    fn read_lower_eigth(&self) -> Result<Vec<i8>, &'static str> {
        let bits = self.read()?;
        Ok(bits[CPU_BITTAGE * 7 / 8..].to_vec())
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

    fn write_bool(&self, value: bool) -> Result<(), RizeError> {
        let mut bits = self.bits.lock().unwrap();
        bits[0] = value as i8;
        Ok(())
    }

    fn write_lower_half(&self, value: Vec<i8>) -> Result<(), RizeError> {
        let mut bits = self.bits.lock().map_err(|_| RizeError {
            type_: RizeErrorType::RegisterWrite,
            message: "Failed to acquire lock for write_lower_half".to_string(),
        })?;
        if value.len() != CPU_BITTAGE / 2 {
            return Err(RizeError {
                type_: RizeErrorType::RegisterWrite,
                message: format!(
                    "Invalid length for write_lower_half: expected {}, got {}",
                    CPU_BITTAGE / 2,
                    value.len()
                ),
            });
        }
        let start_index = CPU_BITTAGE / 2;
        bits[start_index..].copy_from_slice(&value);
        Ok(())
    }

    fn write_lower_quarter(&self, value: Vec<i8>) -> Result<(), RizeError> {
        let mut bits = self.bits.lock().map_err(|_| RizeError {
            type_: RizeErrorType::RegisterWrite,
            message: "Failed to acquire lock for write_lower_quarter"
                .to_string(),
        })?;
        if value.len() != CPU_BITTAGE / 4 {
            return Err(RizeError {
                type_: RizeErrorType::RegisterWrite,
                message: format!(
                    "Invalid length for write_lower_quarter: expected {}, got {}",
                    CPU_BITTAGE / 4,
                    value.len()
                ),
            });
        }
        let start_index = CPU_BITTAGE * 3 / 4;
        bits[start_index..].copy_from_slice(&value);
        Ok(())
    }

    fn write_lower_eigth(&self, value: Vec<i8>) -> Result<(), RizeError> {
        let mut bits = self.bits.lock().map_err(|_| RizeError {
            type_: RizeErrorType::RegisterWrite,
            message: "Failed to acquire lock for write_lower_eigth".to_string(),
        })?;
        if value.len() != CPU_BITTAGE / 8 {
            return Err(RizeError {
                type_: RizeErrorType::RegisterWrite,
                message: format!(
                    "Invalid length for write_lower_eigth: expected {}, got {}",
                    CPU_BITTAGE / 8,
                    value.len()
                ),
            });
        }
        let start_index = CPU_BITTAGE * 7 / 8;
        bits[start_index..].copy_from_slice(&value);
        Ok(())
    }

    /// Reads the u16 value from the register, respecting its current section setting.
    fn read_section_u16(&self) -> Result<u16, RizeError> {
        let bits_result = match self.section {
            'a' => self.read(),
            'b' => self.read_lower_half(),
            'c' => self.read_lower_quarter(),
            'd' => self.read_lower_eigth(),
            invalid_section => {
                return Err(RizeError {
                    type_: RizeErrorType::RegisterRead,
                    message: format!(
                        "Invalid section '{}' found in register during read.",
                        invalid_section
                    ),
                })
            }
        };

        // Map the Result<Vec<i8>, &'static str> to Result<Vec<i8>, RizeError>
        let bits = bits_result.map_err(|e| RizeError {
            type_: RizeErrorType::RegisterRead,
            message: format!(
                "Failed to read section '{}': {}",
                self.section, e
            ),
        })?;

        Ok(bits_to_u16(&bits))
    }

    /// Writes a u16 value to the register, respecting its current section setting.
    fn write_section_u16(&self, value: u16) -> Result<(), RizeError> {
        match self.section {
            'a' => self.store_immediate(value as usize),
            'b' => {
                let bits = u16_to_bits(value, CPU_BITTAGE / 2);
                self.write_lower_half(bits)
            }
            'c' => {
                let bits = u16_to_bits(value, CPU_BITTAGE / 4);
                self.write_lower_quarter(bits)
            }
            'd' => {
                let bits = u16_to_bits(value, CPU_BITTAGE / 8);
                self.write_lower_eigth(bits)
            }
            invalid_section => Err(RizeError {
                type_: RizeErrorType::RegisterWrite,
                message: format!(
                    "Invalid section '{}' found in register during write.",
                    invalid_section
                ),
            }),
        } // This directly returns Result<(), RizeError>
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
    /// Finds the base register, sets its section field, and returns a mutable reference.
    pub fn get(&mut self, original_name: &str) -> Option<&mut Register> {
        let mut lookup_name = original_name.to_string();
        let mut section = 'a'; // Default section

        if let Some(first) = original_name.chars().nth(0) {
            // Make first character check case-insensitive
            if first.to_ascii_lowercase() == 'g' && original_name.len() >= 3 {
                if let Some(third) = original_name.chars().nth(2) {
                    let third_lower = third.to_ascii_lowercase();
                    // Make section check case-insensitive
                    if ['a', 'b', 'c', 'd'].contains(&third_lower) {
                        section = third_lower; // Store lowercase section
                        lookup_name.remove(2);
                    } // else: invalid section char, defaults to 'a'
                }
            } // else: not a 'g' register or too short, defaults to 'a'
        }

        let lookup_key = lookup_name.to_ascii_lowercase();

        // Check for key existence before mutable borrow
        if self.all.contains_key(&lookup_key) {
            // Key exists, so get_mut is safe and returns Some.
            // We can unwrap directly here, although get_mut is still safer.
            let base_register = self.all.get_mut(&lookup_key).unwrap(); // Or use get_mut again if paranoid
            base_register.section = section;
            Some(base_register)
        } else {
            // Key doesn't exist, log immutably.
            warn!(
                "Register key '{}' not found in map. Available keys: {:?}",
                lookup_key,
                self.all.keys() // Immutable borrow here
            );
            None
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
    pub fn write(&mut self, address: u16, data: u16) -> Result<(), RizeError> {
        if address as usize >= MEMORY_SIZE_BYTES {
            return Err(RizeError {
                type_: RizeErrorType::MemoryWrite,
                message: format!(
                    "Memory Address Out Of Range! Addr: {}, Max: {}",
                    address,
                    MEMORY_SIZE_BYTES - 1
                ),
            });
        }

        self.bytes[address as usize] = data;
        Ok(())
    }

    pub fn read(&self, address: u16) -> Result<u16, RizeError> {
        if address as usize >= MEMORY_SIZE_BYTES {
            return Err(RizeError {
                type_: RizeErrorType::MemoryRead,
                message: format!(
                    "Memory Address Out Of Range! Addr: {}, Max: {}",
                    address,
                    MEMORY_SIZE_BYTES - 1
                ),
            });
        }

        Ok(self.bytes[address as usize])
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
    MUL,
    DIV,
    NOT,
    AND,
    OR,
    XOR,
    SHL,
    SHR,
    HALT,
    NOP,
    JMP,
    JIZ,
    JIN,
    WDM,
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
            "MUL" => Ok(OpCode::MUL),
            "DIV" => Ok(OpCode::DIV),
            "NOT" => Ok(OpCode::NOT),
            "WDM" => Ok(OpCode::WDM),
            "AND" => Ok(OpCode::AND),
            "OR" => Ok(OpCode::OR),
            "XOR" => Ok(OpCode::XOR),
            "SHL" => Ok(OpCode::SHL),
            "SHR" => Ok(OpCode::SHR),
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
    MemoryWrite,
    MemoryRead,
    RegisterRead,
    RegisterWrite,
    Display,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RizeError {
    pub type_: RizeErrorType,
    pub message: String,
}

/// Converts a slice of bits (i8) into a u16, zero-extending if necessary.
/// Assumes MSB is at index 0.
fn bits_to_u16(bits: &[i8]) -> u16 {
    let mut value: u16 = 0;
    let len = bits.len();
    let start_bit_index = CPU_BITTAGE.saturating_sub(len); // Target bit index in u16

    for (i, bit) in bits.iter().enumerate() {
        if *bit == 1 {
            let bit_pos = CPU_BITTAGE - 1 - (start_bit_index + i);
            value |= 1 << bit_pos;
        }
    }
    value
}

/// Converts the lower `num_bits` of a u16 into a Vec<i8>.
/// MSB will be at index 0.
fn u16_to_bits(value: u16, num_bits: usize) -> Vec<i8> {
    let mut bits = vec![0i8; num_bits];
    let start_bit_index_u16 = CPU_BITTAGE.saturating_sub(num_bits);

    for i in 0..num_bits {
        // Corresponding bit index in the full u16 (from the left/MSB)
        let u16_idx = start_bit_index_u16 + i;
        // Bit position from the right (LSB=0) in the u16
        let bit_pos_from_lsb = CPU_BITTAGE - 1 - u16_idx;

        if (value >> bit_pos_from_lsb) & 1 == 1 {
            bits[i] = 1;
        }
    }
    bits
}

trait PosNegTrait {
    fn is_negative(self) -> bool;
    fn is_positive(self) -> bool;
}

impl PosNegTrait for i32 {
    fn is_negative(self) -> bool {
        self < 0
    }

    fn is_positive(self) -> bool {
        self >= 0
    }
}
