#[allow(unused_imports, non_upper_case_globals)]
use crate::{
    interpreter::collection::opcode_fn::{ld, mov, st},
    types::{ArgType, Byte, ProgramArg, SystemMemory, DSB},
};
use crate::{
    interpreter::{collection::opcode_fn::and, Interpreter},
    types::{ByteOperations, Registers},
};

use std::collections::HashMap;

use super::{
    opcode_fn::{add, div, mul, sub},
    *,
};

// helper function
fn create_registers() -> Registers {
    let mut registers: Registers = Registers {
        all: HashMap::new(),
    };
    AzmInterpreter {}.setup_registers(&mut registers);

    registers
}

#[test]
fn test_mov_register() {
    let arg1: &ProgramArg = &ProgramArg::register("GAA");
    let arg2: &ProgramArg = &ProgramArg::immediate(42);

    let mut registers = create_registers();

    let mut memory: SystemMemory = SystemMemory::default();

    let result = mov(arg1, arg2, &mut registers, &mut memory);
    if let Err(ref e) = result {
        println!("Error: {:?}", e);
    }
    assert!(result.is_ok());
    assert_eq!(
        registers.get("GAA").unwrap().read(),
        DSB::from_cpu_bittage(42)
    );
}

#[test]
fn test_mov_memory() {
    let arg1 = &ProgramArg::memaddr(69);
    let arg2 = &ProgramArg::immediate(42);

    let mut memory: SystemMemory = SystemMemory::default();

    let mut registers = create_registers();

    let result = mov(arg1, arg2, &mut registers, &mut memory);
    if let Err(ref e) = result {
        println!("Error: {:?}", e);
    }
    assert!(result.is_ok());
    assert_eq!(
        *memory
            .bytes
            .get(&arg1.value.clone().unwrap().as_usize())
            .unwrap()
            .dsb
            .lock()
            .unwrap()
            .as_ref(),
        DSB::from_cpu_bittage(42)
    );
}

#[test]
fn test_add_immediate() {
    let mut registers = create_registers();
    let _ = registers.get("gaa").unwrap().byte.add(8usize.into());

    let arg1 = &ProgramArg::register("gab");
    let arg2 = &ProgramArg::immediate(8);

    assert!(add(arg1, arg2, &mut registers).is_ok());
    assert_eq!(
        *registers
            .get("gaa")
            .unwrap()
            .byte
            .dsb
            .lock()
            .unwrap()
            .as_ref(),
        DSB::from_cpu_bittage(16)
    );
}

#[test]
fn test_add_register() {
    let mut registers = create_registers();
    let _ = registers.get("gaa").unwrap().byte.add(8usize.into());

    let arg1 = &ProgramArg::register("gaa");
    let arg2 = &ProgramArg::register("gba");

    {
        let gba = registers.get("gba").unwrap();
        let _ = gba.write(DSB::from_cpu_bittage(8));

        let gaa = registers.get("gaa").unwrap();
        let _ = gaa.write(DSB::from_cpu_bittage(8));
    }

    assert!(add(arg1, arg2, &mut registers).is_ok());
    assert_eq!(
        registers.get("gaa").unwrap().byte.dsb(),
        DSB::from_cpu_bittage(16)
    );
}

#[test]
fn test_add_memaddr() {
    let mut registers = create_registers();
    let _ = registers.get("gaa").unwrap().byte.add(8usize.into());

    let arg1 = &ProgramArg::register("gab");
    let arg2 = &ProgramArg::memaddr(8);

    assert!(add(arg1, arg2, &mut registers).is_err());
}

#[test]
fn test_sub_immediate() {
    let mut registers = create_registers();

    let arg1 = &ProgramArg::register("gaa");
    let arg2 = &ProgramArg::immediate(3);

    {
        let gaa = registers.get("gaa").unwrap();
        let _ = gaa.write(DSB::from_cpu_bittage(10));
    }

    assert!(sub(arg1, arg2, &mut registers).is_ok());
    assert_eq!(
        registers.get("gaa").unwrap().byte.dsb(),
        DSB::from_cpu_bittage(7)
    );
}

#[test]
fn test_sub_register() {
    let mut registers = create_registers();

    let arg1 = &ProgramArg::register("gaa");
    let arg2 = &ProgramArg::register("gba");

    {
        let gaa = registers.get("gaa").unwrap();
        let _ = gaa.write(DSB::from_cpu_bittage(15));

        let gba = registers.get("gba").unwrap();
        let _ = gba.write(DSB::from_cpu_bittage(5));
    }

    assert!(sub(arg1, arg2, &mut registers).is_ok());
    assert_eq!(
        registers.get("gaa").unwrap().byte.dsb(),
        DSB::from_cpu_bittage(10)
    );
}

#[test]
fn test_sub_memaddr() {
    let mut registers = create_registers();

    let arg1 = &ProgramArg::register("gaa");
    let arg2 = &ProgramArg::memaddr(8);

    assert!(sub(arg1, arg2, &mut registers).is_err());
}

#[test]
fn test_mul_immediate() {
    let mut registers = create_registers();

    let arg1 = &ProgramArg::register("gaa");
    let arg2 = &ProgramArg::immediate(5);

    {
        let gaa = registers.get("gaa").unwrap();
        let _ = gaa.write(DSB::from_cpu_bittage(4));
    }

    assert!(mul(arg1, arg2, &mut registers).is_ok());
    assert_eq!(
        registers.get("gaa").unwrap().byte.dsb(),
        DSB::from_cpu_bittage(20)
    );
}

#[test]
fn test_mul_register() {
    let mut registers = create_registers();

    let arg1 = &ProgramArg::register("gaa");
    let arg2 = &ProgramArg::register("gba");

    {
        let gaa = registers.get("gaa").unwrap();
        let _ = gaa.write(DSB::from_cpu_bittage(6));

        let gba = registers.get("gba").unwrap();
        let _ = gba.write(DSB::from_cpu_bittage(7));
    }

    assert!(mul(arg1, arg2, &mut registers).is_ok());
    assert_eq!(
        registers.get("gaa").unwrap().byte.dsb(),
        DSB::from_cpu_bittage(42)
    );
}

#[test]
fn test_mul_memaddr() {
    let mut registers = create_registers();

    let arg1 = &ProgramArg::register("gaa");
    let arg2 = &ProgramArg::memaddr(8);

    assert!(mul(arg1, arg2, &mut registers).is_err());
}

#[test]
fn test_div_immediate() {
    let mut registers = create_registers();

    let arg1 = &ProgramArg::register("gaa");
    let arg2 = &ProgramArg::immediate(2);

    {
        let gaa = registers.get("gaa").unwrap();
        let _ = gaa.write(DSB::from_cpu_bittage(10));
    }

    assert!(div(arg1, arg2, &mut registers).is_ok());
    assert_eq!(
        registers.get("gaa").unwrap().byte.dsb(),
        DSB::from_cpu_bittage(5)
    );
}

#[test]
fn test_div_register() {
    let mut registers = create_registers();

    let arg1 = &ProgramArg::register("gaa");
    let arg2 = &ProgramArg::register("gba");

    {
        let gaa = registers.get("gaa").unwrap();
        let _ = gaa.write(DSB::from_cpu_bittage(20));

        let gba = registers.get("gba").unwrap();
        let _ = gba.write(DSB::from_cpu_bittage(4));
    }

    assert!(div(arg1, arg2, &mut registers).is_ok());
    assert_eq!(
        registers.get("gaa").unwrap().byte.dsb(),
        DSB::from_cpu_bittage(5)
    );
}

#[test]
fn test_div_memaddr() {
    let mut registers = create_registers();

    let arg1 = &ProgramArg::register("gaa");
    let arg2 = &ProgramArg::memaddr(8);

    assert!(div(arg1, arg2, &mut registers).is_err());
}

#[test]
fn test_div_by_zero() {
    let mut registers = create_registers();

    let arg1 = &ProgramArg::register("gaa");
    let arg2 = &ProgramArg::immediate(0);

    {
        let gaa = registers.get("gaa").unwrap();
        let _ = gaa.write(DSB::from_cpu_bittage(10));
    }

    assert!(div(arg1, arg2, &mut registers).is_err());
}

#[test]
fn test_st() {
    let mut registers = create_registers();
    let mut memory: SystemMemory = SystemMemory::default();

    {
        let mar = registers.get("MAR").unwrap();
        let _ = mar.write(DSB::from_cpu_bittage(42));

        let mdr = registers.get("MDR").unwrap();
        let _ = mdr.write(DSB::from_cpu_bittage(69));
    }

    assert!(st(&mut registers, &mut memory).is_ok());
    assert_eq!(
        memory.bytes.get(&42).unwrap().dsb(),
        DSB::from_cpu_bittage(69)
    );
}

#[test]
fn test_ld() {
    let mut registers = create_registers();
    let mut memory: SystemMemory = SystemMemory::default();

    let dsb = DSB::from_cpu_bittage(420);

    {
        let mar = registers.get("MAR").unwrap();
        let _ = mar.write(DSB::from_cpu_bittage(42));

        let mdr = registers.get("MDR").unwrap();
        let _ = mdr.write(DSB::from_cpu_bittage(123));

        memory.write(42, dsb.clone().into());
    }

    assert!(ld(&mut registers, &mut memory).is_ok());
    assert_eq!(registers.get("MDR").unwrap().read(), dsb);
}

#[test]
fn test_and() {
    let mut registers = create_registers();

    let arg1 = &ProgramArg::register("GAA");
    let arg2 = &ProgramArg::register("GBA");

    let _ = registers.get("GAA").unwrap().write(0b11110011usize);
    let _ = registers.get("GBA").unwrap().write(0b11000011usize);

    assert!(and(arg1, arg2, &mut registers).is_ok());
    let val = registers.get("GAA").unwrap().read();
    println!("Val: {:#b}", &val.as_usize());
    assert_eq!(val, DSB::from_cpu_bittage(0b11000011usize));
}
