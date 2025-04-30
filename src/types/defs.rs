use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use bevy::prelude::*;

#[derive(Debug, PartialEq, Eq)]
pub struct RizeError {
    pub type_: RizeErrorType,
}

/// ### DSB -> Dynamically Sized Byte
/// to allow for runtime adjustment of CPU bittage
#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
pub enum DSB {
    Flag(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
}
#[derive(Debug, Default)]
pub struct Byte {
    pub dsb: Mutex<Arc<DSB>>,
}
#[derive(Debug)]
pub struct Bits {
    pub vec: Vec<u8>,
}
#[derive(Debug)]
pub struct ByteOpResult {
    pub previous: DSB,
}
#[derive(Debug)]
pub struct Flag {
    pub name: String,
    pub value: Byte,
}
#[derive(Debug, Default, Resource)]
pub struct SystemMemory {
    pub bytes: HashMap<usize, Byte>,
}
#[derive(Debug)]
pub struct Register {
    pub name: String,
    pub byte: Byte,
}
#[derive(Debug, Default, Resource)]
pub struct Registers {
    pub all: HashMap<String, Register>,
}
#[derive(Debug, Default)]
pub struct Flags {
    pub all: HashMap<String, Flag>,
}
#[derive(Debug, Default, Resource)]
pub struct ProgramSettings {
    pub autostep: bool,
    pub autostep_lines: usize,
}
#[derive(Debug, Default, Resource)]
pub struct ActiveProgram {
    /// Contents of the entire program
    pub contents: String,
    /// Symbols are stored in a HashMap, where:
    /// - String: the symbol name
    /// - usize: the line number in the program where the symbol is defined
    pub symbols: HashMap<String, usize>,
    /// Last fetched program line
    pub line: String,
    pub opcode: OpCode,
    pub arg1: ArgType,
    pub arg2: ArgType,
    pub arg3: ArgType,
}

#[derive(Debug, PartialEq, Eq)]
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

pub trait ByteOperations {
    fn read(&self) -> Result<DSB, RizeError>;
    fn write(&self, data: DSB) -> Result<ByteOpResult, RizeError>;
    fn add(&self, data: DSB) -> Result<ByteOpResult, RizeError>;
    fn sub(&self, data: DSB) -> Result<ByteOpResult, RizeError>;
    fn mul(&self, data: DSB) -> Result<ByteOpResult, RizeError>;
    fn div(&self, data: DSB) -> Result<ByteOpResult, RizeError>;
}

#[derive(Debug, Default, Clone, Copy)]
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

#[derive(Debug, Default, Clone)]
pub enum ArgType {
    #[default]
    None,
    Error,
    Register(String),
    MemAddr(u16),
    Immediate(u16),
    Symbol(String),
}

// File Stuff //

#[derive(Resource)]
#[allow(dead_code)]
pub struct FileCheckTimer(Timer);

#[derive(Debug, Default, Resource)]
pub struct AzmPrograms(pub Vec<(PathBuf, String)>);
