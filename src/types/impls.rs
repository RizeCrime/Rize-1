use bevy::prelude::*;

use std::{
    collections::HashMap,
    ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Not, Shl, Shr, Sub},
    sync::{Arc, Mutex},
};

use crate::{
    constants::CPU_BITTAGE,
    types::{
        Bits, Byte, ByteOpResult, ByteOperations, OpCode, Register, Registers,
        RizeError, RizeErrorType, DSB,
    },
};

use super::{ArgType, AzmFile, ProgramSettings};

// --- //
// DSB //
// --- //
impl DSB {
    pub fn _test() {}

    /// Helper function to create a DSB from a usize result, matching the size of the original DSB.
    fn from_usize_matching_size(result: usize, original: &DSB) -> DSB {
        match original.get_size() {
            1 => DSB::Flag(result != 0),
            8 => DSB::U8(result as u8),
            16 => DSB::U16(result as u16),
            32 => DSB::U32(result as u32),
            64 => DSB::U64(result as u64),
            128 => DSB::U128(result as u128),
            _ => unreachable!(
                "Invalid bit size encountered: {}",
                original.get_size()
            ),
        }
    }

    /// Creates a DSB from a usize, sized according to CPU_BITTAGE.
    pub fn from_cpu_bittage(value: usize) -> DSB {
        match CPU_BITTAGE {
            1 => DSB::Flag(value != 0), // Treat size 1 as flag
            8 => DSB::U8(value as u8),
            16 => DSB::U16(value as u16),
            32 => DSB::U32(value as u32),
            64 => DSB::U64(value as u64),
            128 => DSB::U128(value as u128),
            _ => panic!(
                "Invalid CPU_BITTAGE ({}) encountered! Choose 1, 8, 16, 32, 64, or 128",
                CPU_BITTAGE
            ),
        }
    }
}
impl Default for DSB {
    fn default() -> Self {
        match CPU_BITTAGE {
            8 => DSB::U8(0),
            16 => DSB::U16(0),
            32 => DSB::U32(0),
            64 => DSB::U64(0),
            128 => DSB::U128(0),
            _ => panic!("Invalid CPU bittage! Choose 8, 16, 32, 64, or 128"),
        }
    }
}
impl From<u8> for DSB {
    fn from(value: u8) -> Self {
        DSB::U8(value)
    }
}
impl From<u16> for DSB {
    fn from(value: u16) -> Self {
        DSB::U16(value)
    }
}
impl From<u32> for DSB {
    fn from(value: u32) -> Self {
        DSB::U32(value)
    }
}
impl From<i32> for DSB {
    fn from(value: i32) -> Self {
        DSB::U32(value as u32)
    }
}
impl From<u64> for DSB {
    fn from(value: u64) -> Self {
        DSB::U64(value)
    }
}
impl From<u128> for DSB {
    fn from(value: u128) -> Self {
        DSB::U128(value)
    }
}
impl From<usize> for DSB {
    fn from(value: usize) -> Self {
        DSB::from_cpu_bittage(value)
    }
}
impl From<Bits> for DSB {
    fn from(value: Bits) -> Self {
        let decimal = value.as_decimal();
        match CPU_BITTAGE {
            8 => DSB::U8(decimal as u8),
            16 => DSB::U16(decimal as u16),
            32 => DSB::U32(decimal as u32),
            64 => DSB::U64(decimal as u64),
            128 => DSB::U128(decimal as u128),
            _ => unreachable!(),
        }
    }
}

impl Add<DSB> for DSB {
    type Output = DSB;

    fn add(self, other: DSB) -> Self::Output {
        let a = self.as_usize();
        let b = other.as_usize();
        let result = a.wrapping_add(b);
        DSB::from_usize_matching_size(result, &self)
    }
}

impl Sub<DSB> for DSB {
    type Output = DSB;

    fn sub(self, other: DSB) -> Self::Output {
        let a = self.as_usize();
        let b = other.as_usize();
        let result = a.wrapping_sub(b);
        DSB::from_usize_matching_size(result, &self)
    }
}

impl Mul<DSB> for DSB {
    type Output = DSB;

    fn mul(self, other: DSB) -> Self::Output {
        let a = self.as_usize();
        let b = other.as_usize();
        let result = a.wrapping_mul(b);
        DSB::from_usize_matching_size(result, &self)
    }
}

impl Div<DSB> for DSB {
    type Output = DSB;

    fn div(self, other: DSB) -> Self::Output {
        let a = self.as_usize();
        let b = other.as_usize();

        if b == 0 {
            // Division by zero returns 0, matching the size of the dividend.
            return DSB::from_usize_matching_size(0, &self);
        }

        let result = a.wrapping_div(b);
        DSB::from_usize_matching_size(result, &self)
    }
}

impl BitAnd<DSB> for DSB {
    type Output = DSB;

    fn bitand(self, other: Self) -> Self::Output {
        let a = self.as_usize();
        let b = other.as_usize();
        let result = a & b;
        DSB::from_usize_matching_size(result, &self)
    }
}

impl Shr<DSB> for DSB {
    type Output = DSB;

    fn shr(self, other: DSB) -> Self::Output {
        let a = self.as_usize();
        // Ensure shift amount is within valid range for usize::shr
        // Rust's shr panics if shift amount >= bits in type.
        // While wrapping_shr exists, usize::shr directly might be slightly more performant
        // if we cap the shift amount appropriately. Let's use wrapping_shr for simplicity
        // and consistency with other ops, though clamping `b` might be another option.
        let b = other.as_usize() as u32; // Cast to u32 for wrapping_shr/shl
        let result = a.wrapping_shr(b);
        DSB::from_usize_matching_size(result, &self)
    }
}

impl Shl<DSB> for DSB {
    type Output = DSB;

    fn shl(self, other: DSB) -> Self::Output {
        let a = self.as_usize();
        let b = other.as_usize() as u32; // Cast to u32 for wrapping_shl
        let result = a.wrapping_shl(b);
        DSB::from_usize_matching_size(result, &self)
    }
}

impl BitOr<DSB> for DSB {
    type Output = DSB;

    fn bitor(self, other: Self) -> Self::Output {
        let a = self.as_usize();
        let b = other.as_usize();
        let result = a | b;
        DSB::from_usize_matching_size(result, &self)
    }
}

impl BitXor<DSB> for DSB {
    type Output = DSB;

    fn bitxor(self, other: Self) -> Self::Output {
        let a = self.as_usize();
        let b = other.as_usize();
        let result = a ^ b;
        DSB::from_usize_matching_size(result, &self)
    }
}

impl Not for DSB {
    type Output = DSB;

    fn not(self) -> Self::Output {
        // Perform NOT based on the specific DSB variant's size
        match self {
            DSB::Flag(f) => DSB::Flag(!f),
            DSB::U8(n) => DSB::U8(!n),
            DSB::U16(n) => DSB::U16(!n),
            DSB::U32(n) => DSB::U32(!n),
            DSB::U64(n) => DSB::U64(!n),
            DSB::U128(n) => DSB::U128(!n),
            DSB::USIZE(n) => DSB::USIZE(!n),
        }
    }
}

impl DSB {
    pub fn get_size(&self) -> usize {
        match self {
            DSB::Flag(_) => 1,
            DSB::U8(_) => 8,
            DSB::U16(_) => 16,
            DSB::U32(_) => 32,
            DSB::U64(_) => 64,
            DSB::U128(_) => 128,
            DSB::USIZE(_) => 64,
        }
    }

    pub fn as_usize(&self) -> usize {
        match self {
            DSB::Flag(f) => *f as usize,
            DSB::U8(n) => *n as usize,
            DSB::U16(n) => *n as usize,
            DSB::U32(n) => *n as usize,
            DSB::U64(n) => *n as usize,
            DSB::U128(n) => *n as usize,
            DSB::USIZE(n) => *n as usize,
        }
    }

    #[deprecated]
    pub fn as_u128(&self) -> u128 {
        match self {
            DSB::Flag(f) => *f as u128,
            DSB::U8(n) => *n as u128,
            DSB::U16(n) => *n as u128,
            DSB::U32(n) => *n as u128,
            DSB::U64(n) => *n as u128,
            DSB::U128(n) => *n,
            DSB::USIZE(n) => *n as u128,
        }
    }

    pub fn as_string(&self) -> String {
        // Bits::from(self.clone()).as_decimal().to_string()
        match self {
            DSB::Flag(f) => f.to_string(),
            DSB::U8(n) => n.to_string(),
            DSB::U16(n) => n.to_string(),
            DSB::U32(n) => n.to_string(),
            DSB::U64(n) => n.to_string(),
            DSB::U128(n) => n.to_string(),
            &DSB::USIZE(n) => n.to_string(),
        }
    }

    pub fn as_utf8(&self) -> String {
        match self {
            DSB::Flag(f) => if *f { "True" } else { "False" }.to_string(),
            DSB::U8(n) => std::char::from_u32(*n as u32)
                .map_or_else(|| String::from("�"), |c| c.to_string()),
            DSB::U16(n) => std::char::from_u32(*n as u32)
                .map_or_else(|| String::from("�"), |c| c.to_string()),
            DSB::U32(n) => std::char::from_u32(*n)
                .map_or_else(|| String::from("�"), |c| c.to_string()),
            DSB::U64(n) => std::char::from_u32(*n as u32)
                .map_or_else(|| String::from("�"), |c| c.to_string()),
            DSB::U128(n) => std::char::from_u32(*n as u32)
                .map_or_else(|| String::from("�"), |c| c.to_string()),
            DSB::USIZE(n) => std::char::from_u32(*n as u32)
                .map_or_else(|| String::from(""), |c| c.to_string()),
        }
    }

    pub fn as_hex(&self) -> String {
        match self {
            DSB::Flag(f) => format!("0x{:x}", *f as u8),
            DSB::U8(n) => format!("0x{:x}", *n),
            DSB::U16(n) => format!("0x{:x}", *n),
            DSB::U32(n) => format!("0x{:x}", *n),
            DSB::U64(n) => format!("0x{:x}", *n),
            DSB::U128(n) => format!("0x{:x}", *n),
            DSB::USIZE(n) => format!("0x{:x}", *n),
        }
    }
}

// ---- //
// Byte //
// ---- //
impl Byte {
    pub fn as_str(&self) -> String {
        self.dsb.lock().unwrap().as_string()
    }

    pub fn as_decimal(&self) -> usize {
        self.dsb.lock().unwrap().as_usize()
    }
}

impl Default for Byte {
    fn default() -> Self {
        Byte {
            size: CPU_BITTAGE,
            dsb: Mutex::new(Arc::new(DSB::default())),
        }
    }
}

impl ByteOperations for Byte {
    fn read(&self) -> Result<DSB, RizeError> {
        Ok((*self.dsb.lock().unwrap()).as_ref().clone())
    }
    fn write(&self, data: DSB) -> Result<ByteOpResult, RizeError> {
        let mut guard = self.dsb.lock().unwrap();
        let prev = guard.as_ref().clone();
        *guard = Arc::new(data);
        Ok(ByteOpResult {
            previous: prev,
            ..Default::default()
        })
    }
    fn add(&self, data: DSB) -> Result<ByteOpResult, RizeError> {
        let mut guard = self.dsb.lock().unwrap();
        let current = guard.as_ref().clone();
        let prev = current.clone();
        *guard = Arc::new(current + data);
        // TODO: Calculate carry flag correctly
        Ok(ByteOpResult {
            previous: prev,
            result: guard.as_ref().clone(), // Set result to the value AFTER operation
            carry: false,                   // Placeholder for carry
        })
    }
    fn sub(&self, data: DSB) -> Result<ByteOpResult, RizeError> {
        let mut guard = self.dsb.lock().unwrap();
        let current = guard.as_ref().clone();
        let prev = current.clone();
        *guard = Arc::new(current - data);
        // TODO: Calculate carry/borrow flag correctly
        let result = ByteOpResult {
            previous: prev,
            result: guard.as_ref().clone(), // Set result to the value AFTER operation
            carry: false,                   // Placeholder for carry/borrow
        };

        debug!("result: {:?}", result);

        Ok(result)
    }
    fn mul(&self, data: DSB) -> Result<ByteOpResult, RizeError> {
        let mut guard = self.dsb.lock().unwrap();
        let current = guard.as_ref().clone();
        let prev = current.clone();
        *guard = Arc::new(current * data);
        // TODO: Calculate carry/overflow flag correctly for multiplication
        Ok(ByteOpResult {
            previous: prev,
            result: guard.as_ref().clone(), // Set result to the value AFTER operation
            carry: false,                   // Placeholder
        })
    }
    fn div(&self, data: DSB) -> Result<ByteOpResult, RizeError> {
        let mut guard = self.dsb.lock().unwrap();
        let current = guard.as_ref().clone();
        let prev = current.clone();
        *guard = Arc::new(current / data); // DSB division handles division by zero
                                           // Division doesn't typically set carry in the same way as add/sub
        Ok(ByteOpResult {
            previous: prev,
            result: guard.as_ref().clone(), // Set result to the value AFTER operation
            carry: false, // Placeholder (usually no carry for division)
        })
    }
    fn bitand(&self, data: DSB) -> Result<ByteOpResult, RizeError> {
        let mut guard = self.dsb.lock().unwrap();
        let current = guard.as_ref().clone();
        let prev = current.clone();
        *guard = Arc::new(current & data);
        Ok(ByteOpResult {
            previous: prev,
            result: guard.as_ref().clone(),
            carry: false,
        })
    }
    fn bitor(&self, data: DSB) -> Result<ByteOpResult, RizeError> {
        let mut guard = self.dsb.lock().unwrap();
        let current = guard.as_ref().clone();
        let prev = current.clone();
        *guard = Arc::new(current | data);
        Ok(ByteOpResult {
            previous: prev,
            result: guard.as_ref().clone(),
            carry: false,
        })
    }
    fn bitxor(&self, data: DSB) -> Result<ByteOpResult, RizeError> {
        let mut guard = self.dsb.lock().unwrap();
        let current = guard.as_ref().clone();
        let prev = current.clone();
        *guard = Arc::new(current ^ data);
        Ok(ByteOpResult {
            previous: prev,
            result: guard.as_ref().clone(),
            carry: false, // XOR doesn't typically have a carry flag in this context
        })
    }
    fn bitnot(&self) -> Result<ByteOpResult, RizeError> {
        let mut guard = self.dsb.lock().unwrap();
        let current = guard.as_ref().clone();
        let prev = current.clone();
        *guard = Arc::new(!current);
        Ok(ByteOpResult {
            previous: prev,
            result: guard.as_ref().clone(),
            carry: false,
        })
    }
    fn bitshl(&self, data: DSB) -> Result<ByteOpResult, RizeError> {
        let mut guard = self.dsb.lock().unwrap();
        let current = guard.as_ref().clone();
        let prev = current.clone();
        let shift_amount = data; // Assuming data holds the shift amount
        *guard = Arc::new(current << shift_amount);
        // TODO: Determine carry flag for shift operations
        Ok(ByteOpResult {
            previous: prev,
            result: guard.as_ref().clone(),
            carry: false, // Placeholder
        })
    }
    fn bitshr(&self, data: DSB) -> Result<ByteOpResult, RizeError> {
        let mut guard = self.dsb.lock().unwrap();
        let current = guard.as_ref().clone();
        let prev = current.clone();
        let shift_amount = data; // Assuming data holds the shift amount
        *guard = Arc::new(current >> shift_amount);
        // TODO: Determine carry flag for shift operations
        Ok(ByteOpResult {
            previous: prev,
            result: guard.as_ref().clone(),
            carry: false, // Placeholder
        })
    }
}

impl From<Bits> for Byte {
    fn from(value: Bits) -> Self {
        let dsb: DSB = value.into();
        Byte {
            size: dsb.get_size(),
            dsb: Mutex::new(Arc::new(dsb)),
        }
    }
}

// ---- //
// Bits //
// ---- //
impl Bits {
    pub fn as_decimal(&self) -> usize {
        let mut decimal = 0;
        let mut power = 1;

        for bit in self.vec.iter().rev() {
            if *bit == 1 {
                decimal += power;
            }
            power *= 2;
        }

        decimal
    }
}

impl From<DSB> for Bits {
    fn from(value: DSB) -> Self {
        let mut bits = Vec::new();
        match value {
            DSB::Flag(f) => {
                // A flag is 1 bit: 1 if true, 0 if false
                bits.push(f as u8);
            }
            DSB::U8(mut n) => {
                for _ in 0..8 {
                    bits.push(n & 1);
                    n >>= 1;
                }
                bits.reverse(); // Ensure correct bit order (MSB first)
            }
            DSB::U16(mut n) => {
                for _ in 0..16 {
                    bits.push((n & 1u16) as u8);
                    n >>= 1;
                }
                bits.reverse();
            }
            DSB::U32(mut n) => {
                for _ in 0..32 {
                    bits.push((n & 1u32) as u8);
                    n >>= 1;
                }
                bits.reverse();
            }
            DSB::U64(mut n) => {
                for _ in 0..64 {
                    bits.push((n & 1u64) as u8);
                    n >>= 1;
                }
                bits.reverse();
            }
            DSB::U128(mut n) => {
                for _ in 0..128 {
                    bits.push((n & 1u128) as u8);
                    n >>= 1;
                }
                bits.reverse();
            }
            // Handle USIZE as u64 for bit representation, adjust if needed
            DSB::USIZE(mut n) => {
                for _ in 0..64 {
                    bits.push((n & 1usize) as u8);
                    n >>= 1;
                }
                bits.reverse();
            }
        }
        // No longer need global reverse here as it's handled per-arm
        Bits { vec: bits }
    }
}

impl From<&Byte> for Bits {
    fn from(value: &Byte) -> Self {
        let dsb: DSB = value.dsb.lock().unwrap().as_ref().clone();
        Bits::from(dsb)
    }
}

// -------- //
// Register //
// -------- //
impl Register {
    pub fn normal<S: Into<String>>(name: S) -> Self {
        Register {
            name: name.into(),
            byte: Byte::default(),
        }
    }
    pub fn flag<S: Into<String>>(name: S) -> Self {
        Register {
            name: name.into(),
            byte: Byte {
                size: 1,
                dsb: Mutex::new(Arc::new(DSB::Flag(false))),
            },
        }
    }

    pub fn read(&self) -> Result<DSB, RizeError> {
        self.byte.read()
    }

    pub fn write<D: Into<DSB>>(&self, data: D) -> Result<(), RizeError> {
        self.byte.write(data.into())?;
        Ok(())
    }

    pub fn inc(&self) -> Result<(), RizeError> {
        self.byte.write(self.byte.read()? + 1.into())?;
        Ok(())
    }
}

// --------- //
// Registers //
// --------- //
impl Registers {
    pub fn insert(&mut self, register: Register) {
        self.all.insert(register.name.clone(), register);
    }
    pub fn all(&self) -> &HashMap<String, Register> {
        &self.all
    }
    pub fn get(&mut self, name: &str) -> Option<&mut Register> {
        if name.is_empty() {
            warn!("Attempted to get register with empty name");
            return None;
        }

        let mut filter = name.to_string().to_ascii_lowercase();

        // Special handling for G registers (e.g. "G00", "G01")
        if name.len() == 3
            && name
                .chars()
                .next()
                .map_or(false, |c| c.to_ascii_lowercase() == 'g')
        {
            // Remove the last digit for G registers
            filter.pop();
        }

        // Check if register exists
        if !self.all.contains_key(&filter) {
            warn!(
                "Register '{}' not found. Available registers: {:?}",
                filter,
                self.all.keys().collect::<Vec<_>>()
            );
            return None;
        }

        // Get the register
        match self.all.get_mut(&filter) {
            Some(register) => Some(register),
            None => {
                error!(
                    "Failed to get register '{}' that was verified to exist",
                    filter
                );
                None
            }
        }
    }
}

// ---------------- //
// Program Settings //
// ---------------- //
impl Default for ProgramSettings {
    fn default() -> Self {
        Self {
            autostep: false,
            autostep_lines: 20,
        }
    }
}

// ------- //
// AzmFile //
// ------- //
impl AzmFile {
    pub fn scan_chunk(&mut self, line_budget: usize) -> Vec<(String, usize)> {
        if self.bytes_scanned == 0 && self.line_starts.is_empty() {
            self.line_starts.push(0);
        }

        let content_slice = &self.original.clone().unwrap()[..];
        let mut line_buffer: Vec<u8> = Vec::new();
        let mut symbols: Vec<(String, usize)> = Vec::new();
        let mut lines_processed = 0;

        while lines_processed < line_budget
            && self.bytes_scanned < self.bytes_to_scan
        {
            let byte = content_slice[self.bytes_scanned];
            line_buffer.push(byte);
            self.bytes_scanned += 1;
            debug!("Line Buffer: {:?}", line_buffer);

            debug!("Byte == '\\n': {:?}", byte == b'\n');
            if byte == b'\n' {
                debug!(
                    "Line Buffer Starts with b\".\": {:?}",
                    line_buffer.starts_with(b".")
                );
                if line_buffer.starts_with(b".") {
                    let symbol_end = if line_buffer.ends_with(b"\r\n") {
                        line_buffer.len() - 2
                    } else {
                        line_buffer.len() - 1
                    };

                    let symbol: (String, usize) = (
                        String::from_utf8(line_buffer[1..symbol_end].to_vec())
                            .unwrap(),
                        self.line_starts.len() - 1,
                    );
                    info!("Symbol: {:?}", symbol);
                    symbols.push(symbol);
                }

                if self.bytes_scanned < self.bytes_to_scan {
                    self.line_starts.push(self.bytes_scanned);
                    self.logical_lines = self.line_starts.len();
                }

                line_buffer.clear();
                lines_processed += 1;
            }
        }

        symbols
    }

    pub fn get_line(&self, index: usize) -> Option<String> {
        // let logical_length: usize = self.pieces.iter().map(|p| p.length).sum();
        if index >= self.logical_lines {
            return None;
        }

        let curr_line_start: usize = self.line_starts[index];

        // Handle the last line case
        let next_line_start: usize = if index + 1 < self.line_starts.len() {
            self.line_starts[index + 1]
        } else {
            // For the last line, use the total size of the file
            self.bytes_to_scan
        };

        let mut line_bytes: Vec<u8> = Vec::new();
        line_bytes.extend_from_slice(
            &self.original.clone().unwrap()
                [curr_line_start..(next_line_start - 1)],
        );

        let line = String::from_utf8(line_bytes).unwrap();

        Some(line)
    }
}

// ------ //
// OpCode //
// ------ //
impl OpCode {
    pub fn as_string(&self) -> String {
        match self {
            OpCode::None => "".to_string(),
            OpCode::LD => "LD".to_string(),
            OpCode::ST => "ST".to_string(),
            OpCode::SWP => "SWP".to_string(),
            OpCode::MOV => "MOV".to_string(),
            OpCode::ADD => "ADD".to_string(),
            OpCode::SUB => "SUB".to_string(),
            OpCode::MUL => "MUL".to_string(),
            OpCode::DIV => "DIV".to_string(),
            OpCode::NOT => "NOT".to_string(),
            OpCode::WDM => "WDM".to_string(),
            OpCode::AND => "AND".to_string(),
            OpCode::OR => "OR".to_string(),
            OpCode::XOR => "XOR".to_string(),
            OpCode::SHL => "SHL".to_string(),
            OpCode::SHR => "SHR".to_string(),
            OpCode::HALT => "HALT".to_string(),
            OpCode::NOP => "NOP".to_string(),
            OpCode::JMP => "JMP".to_string(),
            OpCode::JIZ => "JIZ".to_string(),
            OpCode::JIN => "JIN".to_string(),
        }
    }
}

impl std::str::FromStr for OpCode {
    type Err = RizeError;

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
            _ => Err(RizeError {
                type_: RizeErrorType::Decode("Failed to parse OpCode".into()),
            }),
        }
    }
}

// ------- //
// ArgType //
// ------- //
impl ArgType {
    pub fn as_string(&self) -> String {
        match self {
            ArgType::None => "".to_string(),
            ArgType::Error => "Error".to_string(),
            ArgType::Immediate(i) => i.to_string(),
            ArgType::Register(r) => r.clone(),
            ArgType::MemAddr(m) => m.to_string(),
            ArgType::Symbol(s) => s.clone(),
        }
    }

    /// ### Parsing Rules
    ///
    /// Rules apply in Order, returning the first match.
    ///
    /// 0) if starts with '#'       -> Comment (Ignore)
    /// 1) if only characters       -> Register
    /// 2) if starts with '0x'      -> MemAddr
    /// 3) if is entirely digits    -> Immediate
    /// 4) if starts with '.'       -> Symbol
    pub fn from_string(arg: String) -> Self {
        debug!("types/impls.rs/from_string: {}", arg);

        if arg.is_empty() {
            return ArgType::None;
        }

        // Rule 0: Comments
        if arg.starts_with('#') {
            return ArgType::None;
        }

        // Rule 1: Register
        if arg.chars().all(|c| c.is_alphabetic()) {
            return ArgType::Register(arg.to_string());
        }

        // Rule 2: Memory Address (Hexadecimal)
        if let Some(hex_val) = arg.strip_prefix("0x") {
            if let Ok(addr) = usize::from_str_radix(hex_val, 16) {
                return ArgType::MemAddr(addr);
            }
            return ArgType::Error;
        }

        // Rule 3: Immediate (Decimal)
        if let Ok(imm) = arg.parse::<usize>() {
            return ArgType::Immediate(imm);
        }

        // Rule 4: Symbol
        if let Some(symbol_name) = arg.strip_prefix('.') {
            if !symbol_name.is_empty()
                && symbol_name.chars().all(char::is_alphabetic)
            {
                return ArgType::Symbol(symbol_name.to_string());
            }
            // If it starts with '.' but isn't a valid symbol format
            return ArgType::Error;
        }

        // Default/Error if none of the above match
        ArgType::Error
    }
}
