---
tags:
  - Projects/Project
  - IT/Programming/Code
---

# Rize-1 Design Doc (WIP)

## Instruction Set Architecture (ISA)

The Main Goal of this Project is to make a Learning Resource that I wish I had when I started learning Low-Level CPU stuff.  
As such, Visualisation and Interactivity is the Focus, and Performance is Secondary.  
For the Same Reason, some Limitations of x86asm are removed in order to improve the fun-to-be-had.

### Registers

**Specific Purpose Registers**

- 'pc' -> Program Counter
	- Next Instruction
	- 16-bit
- 'ir' -> Instruction Register
	- Current Instruction
	- 16-bit
- 'mar' -> Memory Address Register
	- Memory Address to be Accessed
	- 16-bit
- 'mdr' -> Memory Data Register
	- Data Read From or Written To the Address in 'mar'
	- 16-bit

**Flag Registers**

- 'fz' -> Zero Flag
- 'fc' -> Carry Flag
- 'fo' -> Overflow Flag
- 'fn' -> Negative Flag
- General Purpose Flags:
	- 'fr-' Flag Register
	- 8-bits
	- each bit can and **has to** be accessed with:
		- bit 1: 'fra'
		- bit 2: 'frb'
		- bit 3: 'frc'
		- bit 4: 'frd'
		- bit 5: 'fre'
		- bit 6: 'frf'
		- bit 7: 'frg'
		- bit 8: 'frh'
	- I know that that doesn't need to be a CPU feature, seems doable by a user...
	- but hey, it's for learning after all

**General Purpose Registers**

The amount of GPRs is undecided for now, but since this will purely be implemented as an Emulator, I should be able to add and/or remove as many as I want. I think, and hope :D 

Update:  
The GPRs are set at Compile Time via a constant for now, but I'd like to add Metadata Support to the AZM files to specify how many GPRs a program needs.  
This could then be used to create the appropriate amount of GPRs at runtime.  

- General Purpose Registers Naming Scheme:
	- 'x--' -> The Designation
		- 'g--' -> General Purpose
	- '-y-' -> The Index of the Designation
		- 'ga-' -> General Purpose Register 1
		- 'gb-' -> General Purpose Register 2
	- '--z' -> The Index of a Sub-Region
		- 'gaa' -> The Full 16 bits of the 'ga' Register
		- 'gab' -> The Last 8 bits of the 'ga' Register
		- 'gac' -> The Last 4 bits of the 'ga' Register
- Examples:
	- 'gcb' denotes the
		- last eight bits ('--b')
		- of the third ('-c-')
		- General Purpose Register ('g')

### Data Types

- Native handling of 16-bit, 8-bit, 4-bit, and 2-bit Data
	- achieved via subdivision of 16-bit Registers
	- and ALU Operations
- Floating Point Numbers are not planned for the MVP

### Instruction Formats

There is no specific Instruction Format (nor traditional Instruction Register for that Matter) for the Rize-1.  
Arguments on the other hand are limited to 16 bits in length.  

| String | 16 bits | 16 bits | 16 bits |
| ------ | ------- | ------- | ------- |
| OPCODE | ARG1    | ARG2    | ARG3    |

- ARG1 is the Target Operand by Default (Results will be written into here)
- ARG3 is Always Optional, unless Specified Otherwise (with a '\*' in Front)

**Memory Related OPCODES**
(This includes Registers)  

| OPCODE | ARG1     | ARG2     | ARG3     |     |
| ------ | -------- | -------- | -------- | --- |
| LD     |          |          |          |     |
| ST     |          |          |          |     |
| SWP    | Register | Register | Register |     |
| MOV    | Any      | Any      |          |     |

_Memory OPCODE Descriptions:_

| OPCODE | Description                         | ARG3 Description                  |
| ------ | ----------------------------------- | --------------------------------- |
| LD     | Loads the Value of MAR into MDR.    |                                   |
| ST     | Stores the Value of MDR into MAR.   |                                   |
| SWP    | Swaps the Contents of ARG1 and ARG2 | Optional Temporary Swap Register. |
| MOV    | Copies ARG2 into ARG1.              |                                   |

**ALU OPCODES**

| OPCODE | ARG1     | ARG2               | ARG3     |
| ------ | -------- | ------------------ | -------- |
| ADD    | Register | Register/Immediate | Register |
| SUB    | Register | Register/Immediate | Register | 

_ALU OPCODE Descriptions:_

| OPCODE | Description                   | ARG3 Description          |
| ------ | ----------------------------- | ------------------------- |
| ADD    | Adds ARG2 to ARG1.            | Optional Target Register. |
| SUB    | Subtracts ARG2 from ARG1      | Optional Target Register. |
|        |                               |                           |

**Bit Operation OPCODES**

| OPCODE | ARG1          | ARG2          | ARG3          |
| ------ | ------------- | ------------- | ------------- |
| NOT    | Type:Register |               |               |
| AND    | Type:Register | Type:Register | Type:Register |
| OR     |               |               |               |
| XOR    |               |               |               |
| SHL    | Type:Register | Type:Number   |               |
| SHR    | Type:Register | Type:Number   |               |

_Bit Operation OPCODE Descriptions:_ 

| OPCODE | Description                   | Optional Description                    | 
| ------ | ----------------------------- | --------------------------------------- |
| NOT    | Negates all the Bits in ARG1. | Optional Target Register.               |
| SHL    | Bitshifts ARG1 Left by One.   | Optionally Specify the Amount to Shift. |
| SHR    | Bitshifts ARG1 Right by One.  | Optionally Specify the Amount to Shift. |

**Control Flow Related OPCODES**  

| OPCODE | ARG1         | ARG2 | ARG3 | Required Flags |
| ------ | ------------ | ---- | ---- | -------------- |
| HALT   |              |      |      |                |
| NOP    |              |      |      |                |
| JMP    | Type:MemAddr |      |      |                |
| JIZ    | Type:MemAddr |      |      | Zero Flag      |
| JIN    | Type:MemAddr |      |      | Sign Flag      |
|        |              |      |      |                |

_Control Flow OPCODE Descriptions:_

| OPCODE | Description                                          |
| ------ | ---------------------------------------------------- |
| HALT   | Stops CPU Execution.                                 |
| NOP    | No Operation. An Empty Instruction.                  |
| JMP    | Jump to ARG1.                                        |
| JIZ    | Jumps to ARG1, if Zero Flag is True.                 |
| JIN    | Jumps to ARG1, if Sign Flag (Negative Flag) is True. |
|        |                                                      |

**Special OPCODES**

| OPCODE | ARG1    | ARG2    | ARG3    |
| ------ | ------- | ------- | ------- |
| WDM    | 8:R,8:G | 8:B,8:A | 8:x,8:y | 

WDM -> Write Display Memory

## Interpreter

Any Interpreter must implement this Trait.  

``` Rust
trait Interpreter {
	pub fn fetch(
		program: ActiveProgram,
		regs: Registers,
	) -> Result<(), RizeError>;
	pub fn decode(program: ActiveProgram) -> Result<(), RizeError>;
	pub fn execute(
		program: &mut ActiveProgram,
		regs: Registers,
		flags: Flags,
		sysmem: SystemMemory,
	) -> Result<(), RizeError>;
}
```

### Fetcher  

The `fetch` method needs to Fetch the Next Line from `program.contents`, based on the line stored in the Program Counter (`regs.get('pc'))`), and store it in `program.line`.  


### Decoder

The `decode` method needs to Split the Line that was just fetched into its Parts, and Parse them into `program.opcode`, and `program.argX` (X can be 1 - 3).  

### Executer

The `execute` method needs to perform the Operation in `program.opcode` with the operands in `program.argX`, using `regs`, `flags`, and `sysmem`.  