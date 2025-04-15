---
tags:
  - Projects/Project
  - IT/Programming/Code
---

# Rize-1 Design Doc (WIP)

## Instruction Set Architecture (ISA)

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

- Native handling of 16-bit, 8-bit, and 4-bit Data
	- achieved via subdivision of 16-bit Registers
	- and ALU Operations
- Floating Point Numbers are not planned for the MVP

### Instruction Formats

Instructions are (8 + 16 + 16 + 16 =) 56 bits long and divided as shown:

| 8 bits | 16 bits | 16 bits | 16 bits |
| ------ | ------- | ------- | ------- |
| OPCODE | ARG1    | ARG2    | ARG3    |

- ARG1 is the Target by Default
- ARG3 is Always Optional, unless Specified Otherwise (with a '\*' in Front)

**Memory Related OPCODES**

| OPCODE | ARG1          | ARG2          | ARG3          |
| ------ | ------------- | ------------- | ------------- |
| LD     |               |               |               |
| ST     |               |               |               |
| SWP    | Type:Register | Type:Register | Type:Register |
| MOV    | Any           | Any           |               | 

_Memory OPCODE Descriptions:_

| OPCODE | Description                         | ARG3 Description                  |
| ------ | ----------------------------------- | --------------------------------- |
| LD     | Loads the Value of MAR into MDR.    |                                   |
| ST     | Stores the Value of MDR into MAR.   |                                   |
| SWP    | Swaps the Contents of ARG1 and ARG2 | Optional Temporary Swap Register. |
| MOV    | Copies ARG2 into ARG1.              |                                   |

**ALU OPCODES**

| OPCODE | ARG1          | ARG2          | ARG3          |
| ------ | ------------- | ------------- | ------------- |
| ADD    | Type:Register | Type:Register | Type:Register |
| SUB    | Type:Register | Type:Register | Type:Register |

_ALU OPCODE Descriptions:_

| OPCODE | Description                   | ARG3 Description          |
| ------ | ----------------------------- | ------------------------- |
| ADD    | Adds ARG2 to ARG1.            | Optional Target Register. |
| SUB    | Subtracts ARG2 from ARG1      | Optional Target Register. |
|        |                               |                           |

**Bit Operation OPCODES**

| OPCODE | ARG1          | ARG2          | ARG3 |
| ------ | ------------- | ------------- | ---- |
| NOT    | Type:Register |               |      |
| AND    | Type:Register | Type:Register |      |
| OR     |               |               |      |
| XOR    |               |               |      |
|        |               |               |      |

_Bit Operation OPCODE Descriptions:_ 

| OPCODE | Description                   | ARG3 Description          |
| ------ | ----------------------------- | ------------------------- |
| NOT    | Negates all the Bits in ARG1. | Optional Target Register. |
|        |                               |                           |

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

| OPCODE | Description                         |
| ------ | ----------------------------------- |
| HALT   | Stops CPU Execution.                |
| NOP    | No Operation. An Empty Instruction. |
|        |                                     |
