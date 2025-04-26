// CPU Bits
pub const CPU_BITTAGE: usize = 16;
pub const N_GENERAL_PURPOSE_REGISTERS: usize = 4;
pub const INSTRUCTION_WIDTH: usize = 56;

// Memory
pub const MEMORY_SIZE_BYTES: usize = 2048;

// Other
pub const DISPLAY_WIDTH: usize = 32;
pub const DISPLAY_HEIGHT: usize = 32;
pub const AZZEMBLY_DIR: &str = "azzembly/";
pub const AUTOSTEP_LINES_PER_FRAME: usize = 10;

// Registers
pub const PROGRAM_COUNTER: &str = "pc";
pub const FLAG_ZERO: &str = "fz";
pub const FLAG_NEGATIVE: &str = "fn";
pub const FLAG_CARRY: &str = "fc";
pub const FLAG_OVERFLOW: &str = "fo";
