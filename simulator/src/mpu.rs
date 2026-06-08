use crate::memory::Memory;
use crate::processor::Processor;

pub const PC_TO_AR: u32 = 1 << 0;
pub const PC_INCR: u32 = 1 << 1;
pub const DR_TO_CR: u32 = 1 << 2;
pub const OPERAND_TO_AR: u32 = 1 << 3;
pub const DR_TO_AR: u32 = 1 << 4;
pub const MEM_TO_DR: u32 = 1 << 5;
pub const DR_TO_MEM: u32 = 1 << 6;
pub const DR_TO_AC: u32 = 1 << 7;
pub const ALU_TO_AC: u32 = 1 << 8;
pub const AC_TO_DR: u32 = 1 << 9;
pub const PC_TO_DR: u32 = 1 << 10;
pub const SP_TO_DR: u32 = 1 << 11;
pub const SP_TO_AR: u32 = 1 << 12;
pub const SP_INCR: u32 = 1 << 13;
pub const SP_DECR: u32 = 1 << 14;
pub const DR_TO_PC: u32 = 1 << 15;
pub const ALU_TO_PC: u32 = 1 << 16;
pub const DR_TO_DATA_BUS: u32 = 1 << 17;
pub const IMM_TO_DATA_BUS: u32 = 1 << 18;
pub const ZERO_TO_DATA_BUS: u32 = 1 << 19;
pub const UPDATE_FLAGS: u32 = 1 << 20;
pub const DONE: u32 = 1 << 21;
pub const HALT: u32 = 1 << 29;

pub const fn alu_op(op: u32) -> u32 {
    op << 25
}

pub const fn cond(c: u32) -> u32 {
    c << 22
}

pub const ALU_PASS: u32 = alu_op(0);
pub const ALU_ADD: u32 = alu_op(1);
pub const ALU_SUB: u32 = alu_op(2);
pub const ALU_MUL: u32 = alu_op(3);
pub const ALU_DIV: u32 = alu_op(4);
pub const ALU_MOD: u32 = alu_op(5);
pub const ALU_AND: u32 = alu_op(6);
pub const ALU_OR: u32 = alu_op(7);
pub const ALU_XOR: u32 = alu_op(8);
pub const ALU_NEG: u32 = alu_op(9);
pub const ALU_NOT: u32 = alu_op(10);

pub const COND_ALWAYS: u32 = cond(0);
pub const COND_Z: u32 = cond(1);
pub const COND_NZ: u32 = cond(2);
pub const COND_N: u32 = cond(3);
pub const COND_NN: u32 = cond(4);

pub const OP_LD: usize = 0x01;
pub const OP_ST: usize = 0x02;
pub const OP_ADD: usize = 0x03;
pub const OP_SUB: usize = 0x04;
pub const OP_MUL: usize = 0x05;
pub const OP_DIV: usize = 0x06;
pub const OP_MOD: usize = 0x07;
pub const OP_CMP: usize = 0x08;
pub const OP_NEG: usize = 0x09;
pub const OP_NOT: usize = 0x0A;
pub const OP_AND: usize = 0x0B;
pub const OP_OR: usize = 0x0C;
pub const OP_XOR: usize = 0x0D;
pub const OP_JMP: usize = 0x0E;
pub const OP_JZS: usize = 0x0F;
pub const OP_JZC: usize = 0x10;
pub const OP_JNS: usize = 0x11;
pub const OP_JNC: usize = 0x12;
pub const OP_CALL: usize = 0x13;
pub const OP_RET: usize = 0x14;
pub const OP_HALT: usize = 0x15;

pub const MODE_DIRECT: usize = 0;
pub const MODE_IMM: usize = 1;
pub const MODE_INDIRECT: usize = 2;

const NUM_OPCODES: usize = 22;
const NUM_MODES: usize = 4;
const SLOTS_PER_INSTR: usize = 6;
const MICROCODE_SIZE: usize = NUM_OPCODES * NUM_MODES * SLOTS_PER_INSTR;

const MICROCODE: [u32; MICROCODE_SIZE] = build_microcode();

const fn micro_addr(opcode: usize, mode: usize, slot: usize) -> usize {
    (opcode * NUM_MODES + mode) * SLOTS_PER_INSTR + slot
}

const fn build_microcode() -> [u32; MICROCODE_SIZE] {
    let mut rom = [0u32; MICROCODE_SIZE];
    let mut op = 0usize;
    while op < NUM_OPCODES {
        let mut mode = 0usize;
        while mode < NUM_MODES {
            let base = (op * NUM_MODES + mode) * SLOTS_PER_INSTR;
            rom[base] = PC_TO_AR | MEM_TO_DR;
            rom[base + 1] = DR_TO_CR | PC_INCR;

            let empty = HALT | DONE;

            match op {
                OP_LD => match mode {
                    MODE_DIRECT => {
                        rom[base + 2] = OPERAND_TO_AR | MEM_TO_DR;
                        rom[base + 3] = DR_TO_DATA_BUS | ALU_PASS | ALU_TO_AC | UPDATE_FLAGS | DONE;
                    }
                    MODE_IMM => {
                        rom[base + 2] =
                            IMM_TO_DATA_BUS | ALU_PASS | ALU_TO_AC | UPDATE_FLAGS | DONE;
                    }
                    MODE_INDIRECT => {
                        rom[base + 2] = OPERAND_TO_AR | MEM_TO_DR;
                        rom[base + 3] = DR_TO_AR | MEM_TO_DR;
                        rom[base + 4] = DR_TO_DATA_BUS | ALU_PASS | ALU_TO_AC | UPDATE_FLAGS | DONE;
                    }
                    _ => {
                        rom[base + 2] = empty;
                    }
                },
                OP_ST => match mode {
                    MODE_DIRECT => {
                        rom[base + 2] = AC_TO_DR;
                        rom[base + 3] = OPERAND_TO_AR | DR_TO_MEM | DONE;
                    }
                    MODE_INDIRECT => {
                        rom[base + 2] = OPERAND_TO_AR | MEM_TO_DR;
                        rom[base + 3] = DR_TO_AR;
                        rom[base + 4] = AC_TO_DR;
                        rom[base + 5] = DR_TO_MEM | DONE;
                    }
                    _ => {
                        rom[base + 2] = empty;
                    }
                },
                OP_ADD | OP_SUB | OP_MUL | OP_DIV | OP_MOD | OP_AND | OP_OR | OP_XOR => {
                    let alu = match op {
                        OP_ADD => ALU_ADD,
                        OP_SUB => ALU_SUB,
                        OP_MUL => ALU_MUL,
                        OP_DIV => ALU_DIV,
                        OP_MOD => ALU_MOD,
                        OP_AND => ALU_AND,
                        OP_OR => ALU_OR,
                        _ => ALU_XOR,
                    };
                    match mode {
                        MODE_DIRECT => {
                            rom[base + 2] = OPERAND_TO_AR | MEM_TO_DR;
                            rom[base + 3] = DR_TO_DATA_BUS | alu | ALU_TO_AC | UPDATE_FLAGS | DONE;
                        }
                        MODE_IMM => {
                            rom[base + 2] = IMM_TO_DATA_BUS | alu | ALU_TO_AC | UPDATE_FLAGS | DONE;
                        }
                        MODE_INDIRECT => {
                            rom[base + 2] = OPERAND_TO_AR | MEM_TO_DR;
                            rom[base + 3] = DR_TO_AR | MEM_TO_DR;
                            rom[base + 4] = DR_TO_DATA_BUS | alu | ALU_TO_AC | UPDATE_FLAGS | DONE;
                        }
                        _ => {
                            rom[base + 2] = empty;
                        }
                    }
                }
                OP_CMP => match mode {
                    MODE_DIRECT => {
                        rom[base + 2] = OPERAND_TO_AR | MEM_TO_DR;
                        rom[base + 3] = DR_TO_DATA_BUS | ALU_SUB | UPDATE_FLAGS | DONE;
                    }
                    MODE_IMM => {
                        rom[base + 2] = IMM_TO_DATA_BUS | ALU_SUB | UPDATE_FLAGS | DONE;
                    }
                    MODE_INDIRECT => {
                        rom[base + 2] = OPERAND_TO_AR | MEM_TO_DR;
                        rom[base + 3] = DR_TO_AR | MEM_TO_DR;
                        rom[base + 4] = DR_TO_DATA_BUS | ALU_SUB | UPDATE_FLAGS | DONE;
                    }
                    _ => {
                        rom[base + 2] = empty;
                    }
                },
                OP_NEG => rom[base + 2] = ALU_NEG | ALU_TO_AC | UPDATE_FLAGS | DONE,

                OP_NOT => rom[base + 2] = ALU_NOT | ALU_TO_AC | UPDATE_FLAGS | DONE,
                OP_JMP => match mode {
                    MODE_DIRECT | MODE_IMM => {
                        rom[base + 2] = ALU_PASS | IMM_TO_DATA_BUS | ALU_TO_PC | COND_ALWAYS | DONE;
                    }
                    MODE_INDIRECT => {
                        rom[base + 2] = OPERAND_TO_AR | MEM_TO_DR;
                        rom[base + 3] = DR_TO_DATA_BUS | ALU_PASS | ALU_TO_PC | COND_ALWAYS | DONE;
                    }
                    _ => {
                        rom[base + 2] = empty;
                    }
                },
                OP_JZS => match mode {
                    MODE_DIRECT | MODE_IMM => {
                        rom[base + 2] = ALU_PASS | IMM_TO_DATA_BUS | ALU_TO_PC | COND_Z | DONE;
                    }
                    MODE_INDIRECT => {
                        rom[base + 2] = OPERAND_TO_AR | MEM_TO_DR;
                        rom[base + 3] = DR_TO_DATA_BUS | ALU_PASS | ALU_TO_PC | COND_Z | DONE;
                    }
                    _ => {
                        rom[base + 2] = empty;
                    }
                },
                OP_JZC => match mode {
                    MODE_DIRECT | MODE_IMM => {
                        rom[base + 2] = ALU_PASS | IMM_TO_DATA_BUS | ALU_TO_PC | COND_NZ | DONE;
                    }
                    MODE_INDIRECT => {
                        rom[base + 2] = OPERAND_TO_AR | MEM_TO_DR;
                        rom[base + 3] = DR_TO_DATA_BUS | ALU_PASS | ALU_TO_PC | COND_NZ | DONE;
                    }
                    _ => {
                        rom[base + 2] = empty;
                    }
                },
                OP_JNS => match mode {
                    MODE_DIRECT | MODE_IMM => {
                        rom[base + 2] = ALU_PASS | IMM_TO_DATA_BUS | ALU_TO_PC | COND_N | DONE;
                    }
                    MODE_INDIRECT => {
                        rom[base + 2] = OPERAND_TO_AR | MEM_TO_DR;
                        rom[base + 3] = DR_TO_DATA_BUS | ALU_PASS | ALU_TO_PC | COND_N | DONE;
                    }
                    _ => {
                        rom[base + 2] = empty;
                    }
                },
                OP_JNC => match mode {
                    MODE_DIRECT | MODE_IMM => {
                        rom[base + 2] = ALU_PASS | IMM_TO_DATA_BUS | ALU_TO_PC | COND_NN | DONE;
                    }
                    MODE_INDIRECT => {
                        rom[base + 2] = OPERAND_TO_AR | MEM_TO_DR;
                        rom[base + 3] = DR_TO_DATA_BUS | ALU_PASS | ALU_TO_PC | COND_NN | DONE;
                    }
                    _ => {
                        rom[base + 2] = empty;
                    }
                },
                OP_CALL => match mode {
                    MODE_DIRECT | MODE_IMM => {
                        rom[base + 2] = PC_TO_DR | SP_INCR | SP_TO_AR;
                        rom[base + 3] = DR_TO_MEM;
                        rom[base + 4] = ALU_PASS | IMM_TO_DATA_BUS | ALU_TO_PC | DONE;
                    }
                    _ => {
                        rom[base + 2] = empty;
                    }
                },
                OP_RET => {
                    rom[base + 2] = SP_DECR;
                    rom[base + 3] = SP_TO_AR | MEM_TO_DR;
                    rom[base + 4] = DR_TO_PC | DONE;
                }
                OP_HALT => {
                    rom[base + 2] = HALT | DONE;
                }
                _ => {
                    rom[base + 2] = empty;
                }
            }

            mode += 1;
        }
        op += 1;
    }
    rom
}

const SIGNAL_NAMES: [(u32, &str); 22] = [
    (1 << 0, "PC_TO_AR"),
    (1 << 1, "PC_INCR"),
    (1 << 2, "DR_TO_CR"),
    (1 << 3, "OPERAND_TO_AR"),
    (1 << 4, "DR_TO_AR"),
    (1 << 5, "MEM_TO_DR"),
    (1 << 6, "DR_TO_MEM"),
    (1 << 7, "DR_TO_AC"),
    (1 << 8, "ALU_TO_AC"),
    (1 << 9, "AC_TO_DR"),
    (1 << 10, "PC_TO_DR"),
    (1 << 11, "SP_TO_DR"),
    (1 << 12, "SP_TO_AR"),
    (1 << 13, "SP_INCR"),
    (1 << 14, "SP_DECR"),
    (1 << 15, "DR_TO_PC"),
    (1 << 16, "ALU_TO_PC"),
    (1 << 17, "DR_TO_DATA_BUS"),
    (1 << 18, "IMM_TO_DATA_BUS"),
    (1 << 19, "ZERO_TO_DATA_BUS"),
    (1 << 20, "UPDATE_FLAGS"),
    (1 << 21, "DONE"),
];

const ALU_OP_NAMES: [&str; 11] = [
    "PASS", "ADD", "SUB", "MUL", "DIV", "MOD", "AND", "OR", "XOR", "NEG", "NOT",
];

const COND_NAMES: [&str; 5] = ["ALWAYS", "Z", "NZ", "N", "NN"];

pub fn fmt_micro(micro: u32) -> String {
    let mut parts: Vec<&str> = Vec::new();

    if micro & HALT != 0 {
        parts.push("HALT");
    }

    for &(mask, name) in &SIGNAL_NAMES {
        if micro & mask != 0 {
            parts.push(name);
        }
    }

    let alu_op_val = (micro >> 25) & 0xF;
    if alu_op_val <= 10 {
        parts.push(ALU_OP_NAMES[alu_op_val as usize]);
    }

    let cond_val = (micro >> 22) & 0x7;
    if cond_val <= 4 && cond_val != 0 {
        parts.push(COND_NAMES[cond_val as usize]);
    }

    parts.join("|")
}

pub fn disassemble(cr: u32) -> String {
    let opcode = ((cr >> 27) & 0x1F) as usize;
    let mode = ((cr >> 25) & 0x3) as usize;
    let operand = cr & 0x1FFFFFF;

    let name = match opcode {
        OP_LD => "LD",
        OP_ST => "ST",
        OP_ADD => "ADD",
        OP_SUB => "SUB",
        OP_MUL => "MUL",
        OP_DIV => "DIV",
        OP_MOD => "MOD",
        OP_CMP => "CMP",
        OP_NEG => "NEG",
        OP_NOT => "NOT",
        OP_AND => "AND",
        OP_OR => "OR",
        OP_XOR => "XOR",
        OP_JMP => "JMP",
        OP_JZS => "JZS",
        OP_JZC => "JZC",
        OP_JNS => "JNS",
        OP_JNC => "JNC",
        OP_CALL => "CALL",
        OP_RET => "RET",
        OP_HALT => "HALT",
        _ => "???",
    };

    match mode {
        MODE_DIRECT => format!("{} [{:#X}]", name, operand),
        MODE_IMM => {
            let imm = ((operand as i32) << 7) >> 7;
            format!("{} #{}", name, imm)
        }
        MODE_INDIRECT => format!("{} [[{:#X}]]", name, operand),
        _ => name.to_string(),
    }
}

pub struct Mpu {
    pub micro_pc: u8,
    pub current_micro: u32,
    pub logs: String,
}

impl Default for Mpu {
    fn default() -> Self {
        Self::new()
    }
}

impl Mpu {
    pub fn new() -> Self {
        Mpu {
            micro_pc: 0,
            current_micro: MICROCODE[micro_addr(0, 0, 0)],
            logs: String::new(),
        }
    }

    pub fn fetch(&mut self, opcode: usize, mode: usize) {
        self.current_micro = MICROCODE[micro_addr(opcode, mode, self.micro_pc as usize)];
    }

    pub fn tick(&mut self, proc: &mut Processor, memory: &mut Memory) {
        memory.tick();

        proc.execute_microcommand(self.current_micro);

        if self.current_micro & MEM_TO_DR != 0 {
            if memory.is_done() {
                proc.dr = memory.collect();
            } else if !memory.is_busy() {
                memory.read_start(proc.ar);
                if memory.is_done() {
                    proc.dr = memory.collect();
                }
            }
        }

        if self.current_micro & DR_TO_MEM != 0 {
            if memory.is_done() {
                memory.clear_done();
            } else if !memory.is_busy() {
                memory.write_start(proc.ar, proc.dr);
            }
        }

        if memory.is_done() {
            proc.dr = memory.collect();
        }

        if self.current_micro & HALT != 0 {
            proc.run = false;
        }

        let mem_busy = memory.is_busy();

        let trace_tick = proc.tick_count;
        let trace_upc = self.micro_pc;
        let trace_micro = self.current_micro;
        let trace_cr = proc.cr;
        let trace_pc = proc.pc;
        let trace_sp = proc.sp;
        let trace_ac = proc.ac;
        let trace_ar = proc.ar;
        let trace_dr = proc.dr;
        let trace_mem = if mem_busy { "busy" } else { "idle" };

        if !mem_busy && self.current_micro & DONE != 0 {
            self.micro_pc = 0;
        } else if !mem_busy {
            self.micro_pc += 1;
        }

        self.fetch(proc.opcode(), proc.mode());

        proc.tick_count += 1;

        use std::fmt::Write;
        let micro_str = fmt_micro(trace_micro);
        let _ = writeln!(
            self.logs,
            "TICK {:4} | μPC {} | {:50} | {:15} | PC={:<4} SP={:<4} AC={:<6} AR={:<4} DR={:08X} mem={}",
            trace_tick,
            trace_upc,
            micro_str,
            disassemble(trace_cr),
            trace_pc,
            trace_sp,
            trace_ac,
            trace_ar,
            trace_dr,
            trace_mem,
        );
    }

    pub fn reset(&mut self) {
        self.micro_pc = 0;
        self.current_micro = MICROCODE[micro_addr(0, 0, 0)];
    }
}
