use std::{
    cell::RefCell,
    collections::HashMap,
    ops::{Add, Div, Mul, Sub},
};

use bevy::prelude::*;

/// ### DSB -> Dynamically Sized Byte
/// to allow for runtime adjustment of CPU bittage
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DSB {
    Flag(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
}
#[derive(Debug, PartialEq, Eq)]
pub struct RizeError {
    type_: RizeErrorType,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Byte {
    dsb: RefCell<DSB>,
}
#[derive(Debug)]
pub struct ByteOpResult {
    previous: DSB,
}
#[derive(Debug)]
pub struct Register {
    id: &'static str,
    byte: Byte,
}
pub struct Flag {
    value: Byte,
}
pub struct SystemMemory {
    bytes: HashMap<usize, Byte>,
}
pub struct Registers {
    all: HashMap<String, Register>,
}
pub struct Flags {
    all: HashMap<String, Flag>,
}

pub enum RizeErrorType {
    Fetch(String),
    Decode(String),
    Execute(String),
    Display(String),
    MemoryRead(String),
    MemoryWrite(String),
    RegisterRead(String),
    RegisterWrite(String),
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

impl Add<DSB> for DSB {
    type Output = DSB;

    fn add(self, other: DSB) -> Self::Output {
        let a = self.to_u128();
        let b = other.to_u128();

        let result = a + b;

        self.from_u128(result)
    }
}

impl Sub<DSB> for DSB {
    type Output = DSB;

    fn sub(self, other: DSB) -> Self::Output {
        let a = self.to_u128();
        let b = other.to_u128();

        let result = a - b;

        self.from_u128(result)
    }
}

impl Mul<DSB> for DSB {
    type Output = DSB;

    fn mul(self, other: DSB) -> Self::Output {
        let a = self.to_u128();
        let b = other.to_u128();

        let result = a * b;

        self.from_u128(result)
    }
}

impl Div<DSB> for DSB {
    type Output = DSB;

    fn div(self, other: DSB) -> Self::Output {
        let a = self.to_u128();
        let b = other.to_u128();

        let result = a / b;

        self.from_u128(result)
    }
}

impl DSB {
    fn to_u128(&self) -> u128 {
        match self {
            DSB::Flag(f) => *f as u128,
            DSB::U8(n) => *n as u128,
            DSB::U16(n) => *n as u128,
            DSB::U32(n) => *n as u128,
            DSB::U64(n) => *n as u128,
            DSB::U128(n) => *n,
        }
    }

    fn from_u128(&self, value: u128) -> DSB {
        match self {
            DSB::Flag(_) => DSB::Flag(value != 0),
            DSB::U8(_) => DSB::U8((value & 0xFF) as u8),
            DSB::U16(_) => DSB::U16((value & 0xFFFF) as u16),
            DSB::U32(_) => DSB::U32((value & 0xFFFFFFFF) as u32),
            DSB::U64(_) => DSB::U64((value & 0xFFFFFFFFFFFFFFFF) as u64),
            DSB::U128(_) => DSB::U128(value),
        }
    }
}

pub trait ByteOperations {
    fn read(&self) -> Result<DSB, RizeError>;
    fn write(&self, data: DSB) -> Result<ByteOpResult, RizeError>;
    fn add(&self, data: DSB) -> Result<ByteOpResult, RizeError>;
    fn sub(&self, data: DSB) -> Result<ByteOpResult, RizeError>;
    fn mul(&self, data: DSB) -> Result<ByteOpResult, RizeError>;
    fn div(&self, data: DSB) -> Result<ByteOpResult, RizeError>;
}

impl ByteOperations for Byte {
    fn read(&self) -> Result<DSB, RizeError> {
        Ok(self.dsb.borrow().clone())
    }
    fn write(&self, data: DSB) -> Result<ByteOpResult, RizeError> {
        let prev = self.dsb.replace(data);
        Ok(ByteOpResult { previous: prev })
    }
    fn add(&self, data: DSB) -> Result<ByteOpResult, RizeError> {
        let current: DSB = self.dsb.borrow().clone();
        let result: DSB = current + data;
        let prev = self.dsb.replace(result);
        Ok(ByteOpResult { previous: prev })
    }
    fn sub(&self, data: DSB) -> Result<ByteOpResult, RizeError> {
        let current: DSB = self.dsb.borrow().clone();
        let result: DSB = current - data;
        let prev = self.dsb.replace(result);
        Ok(ByteOpResult { previous: prev })
    }
    fn mul(&self, data: DSB) -> Result<ByteOpResult, RizeError> {
        let current: DSB = self.dsb.borrow().clone();
        let result: DSB = current * data;
        let prev = self.dsb.replace(result);
        Ok(ByteOpResult { previous: prev })
    }
    fn div(&self, data: DSB) -> Result<ByteOpResult, RizeError> {
        let current: DSB = self.dsb.borrow().clone();
        let result: DSB = current / data;
        let prev = self.dsb.replace(result);
        Ok(ByteOpResult { previous: prev })
    }
}
