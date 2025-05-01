use std::{
    collections::HashMap,
    ops::{Add, BitAnd, Div, Mul, Shl, Shr, Sub},
    sync::{Arc, Mutex},
};

use crate::{
    constants::CPU_BITTAGE,
    types::{
        Bits, Byte, ByteOpResult, ByteOperations, OpCode, Register, Registers,
        RizeError, RizeErrorType, DSB,
    },
};

use super::ArgType;

// --- //
// DSB //
// --- //
impl DSB {
    pub fn _test() {}
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
        let a = self.as_u128();
        let b = other.as_u128();

        let result = a + b;

        result.into()
    }
}

impl Sub<DSB> for DSB {
    type Output = DSB;

    fn sub(self, other: DSB) -> Self::Output {
        let a = self.as_u128();
        let b = other.as_u128();

        let result = a - b;

        result.into()
    }
}

impl Mul<DSB> for DSB {
    type Output = DSB;

    fn mul(self, other: DSB) -> Self::Output {
        let a = self.as_u128();
        let b = other.as_u128();

        let result = a * b;

        result.into()
    }
}

impl Div<DSB> for DSB {
    type Output = DSB;

    fn div(self, other: DSB) -> Self::Output {
        let a = self.as_u128();
        let b = other.as_u128();

        let result = a / b;

        result.into()
    }
}

impl BitAnd<DSB> for DSB {
    type Output = DSB;

    fn bitand(self, other: Self) -> Self::Output {
        let a = self.as_u128();
        let b = other.as_u128();

        let result = a & b;

        result.into()
    }
}

impl Shr<DSB> for DSB {
    type Output = DSB;
    
    fn shr(self, other: DSB) -> Self::Output {
        let a = self.as_u128();
        let b = other.as_u128();

        let result = a >> b;

        result.into()
    }
}

impl Shl<DSB> for DSB {
    type Output = DSB;
    
    fn shl(self, other: DSB) -> Self::Output {
        let a = self.as_u128();
        let b = other.as_u128();

        let result = a << b;

        result.into()
    }
}

impl DSB {
    pub fn as_u128(&self) -> u128 {
        match self {
            DSB::Flag(f) => *f as u128,
            DSB::U8(n) => *n as u128,
            DSB::U16(n) => *n as u128,
            DSB::U32(n) => *n as u128,
            DSB::U64(n) => *n as u128,
            DSB::U128(n) => *n,
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            DSB::Flag(f) => f.clone().to_string(),
            DSB::U8(n) => n.clone().to_string(),
            DSB::U16(n) => n.clone().to_string(),
            DSB::U32(n) => n.clone().to_string(),
            DSB::U64(n) => n.clone().to_string(),
            DSB::U128(n) => n.clone().to_string(),
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
        }
    }

    pub fn as_hex(&self) -> String {
        match self {
            DSB::Flag(f) => format!("{:x}", *f as u8),
            DSB::U8(n) => format!("{:x}", *n),
            DSB::U16(n) => format!("{:x}", *n),
            DSB::U32(n) => format!("{:x}", *n),
            DSB::U64(n) => format!("{:x}", *n),
            DSB::U128(n) => format!("{:x}", *n),
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

    pub fn as_decimal(&self) -> u128 {
        self.dsb.lock().unwrap().as_u128()
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
        Ok(ByteOpResult { previous: prev })
    }
    fn add(&self, data: DSB) -> Result<ByteOpResult, RizeError> {
        let mut guard = self.dsb.lock().unwrap();
        let current = guard.as_ref().clone();
        let prev = current.clone();
        *guard = Arc::new(current + data);
        Ok(ByteOpResult { previous: prev })
    }
    fn sub(&self, data: DSB) -> Result<ByteOpResult, RizeError> {
        let mut guard = self.dsb.lock().unwrap();
        let current = guard.as_ref().clone();
        let prev = current.clone();
        *guard = Arc::new(current - data);
        Ok(ByteOpResult { previous: prev })
    }
    fn mul(&self, data: DSB) -> Result<ByteOpResult, RizeError> {
        let mut guard = self.dsb.lock().unwrap();
        let current = guard.as_ref().clone();
        let prev = current.clone();
        *guard = Arc::new(current * data);
        Ok(ByteOpResult { previous: prev })
    }
    fn div(&self, data: DSB) -> Result<ByteOpResult, RizeError> {
        let mut guard = self.dsb.lock().unwrap();
        let current = guard.as_ref().clone();
        let prev = current.clone();
        *guard = Arc::new(current / data);
        Ok(ByteOpResult { previous: prev })
    }
}

impl From<Bits> for Byte {
    fn from(value: Bits) -> Self {
        let dsb: DSB = value.into();
        Byte {
            dsb: Mutex::new(Arc::new(dsb)),
        }
    }
}

// ---- //
// Bits //
// ---- //
impl Bits {
    pub fn as_decimal(&self) -> u128 {
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
            DSB::U8(mut n) => {
                for _ in 0..8 {
                    bits.push(n & 1);
                    n >>= 1;
                }
            }
            DSB::U16(mut n) => {
                for _ in 0..16 {
                    bits.push((n & 1u16) as u8);
                    n >>= 1;
                }
            }
            DSB::U32(mut n) => {
                for _ in 0..32 {
                    bits.push((n & 1u32) as u8);
                    n >>= 1;
                }
            }
            DSB::U64(mut n) => {
                for _ in 0..64 {
                    bits.push((n & 1u64) as u8);
                    n >>= 1;
                }
            }
            DSB::U128(mut n) => {
                for _ in 0..128 {
                    bits.push((n & 1u128) as u8);
                    n >>= 1;
                }
            }
            _ => {}
        }
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
        self.all.get_mut(name)
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
            if let Ok(addr) = u16::from_str_radix(hex_val, 16) {
                return ArgType::MemAddr(addr);
            }
            return ArgType::Error;
        }

        // Rule 3: Immediate (Decimal)
        if let Ok(imm) = arg.parse::<u16>() {
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