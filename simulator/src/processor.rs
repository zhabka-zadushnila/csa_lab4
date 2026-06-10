use crate::mpu::{
    AC_TO_DR, ALU_TO_AC, ALU_TO_PC, DR_TO_AC, DR_TO_AR, DR_TO_CR, DR_TO_DATA_BUS, DR_TO_PC,
    IMM_TO_DATA_BUS, OPERAND_TO_AR, PC_INCR, PC_TO_AR, PC_TO_DR, SP_DECR, SP_INCR, SP_TO_AR,
    SP_TO_DR, UPDATE_FLAGS, ZERO_TO_DATA_BUS,
};

pub struct Processor {
    pub pc: u32,
    pub sp: u32,
    pub ac: i32,
    pub cr: u32,
    pub ar: u32,
    pub dr: i32,
    pub data_bus: i32,
    pub z: bool,
    pub n: bool,
    pub alu_op: u8,
    pub run: bool,
    pub tick_count: u64,
}

impl Default for Processor {
    fn default() -> Self {
        Self::new()
    }
}

impl Processor {
    pub fn new() -> Self {
        Processor {
            pc: 0,
            sp: 0,
            ac: 0,
            cr: 0,
            ar: 0,
            dr: 0,
            data_bus: 0,
            z: false,
            n: false,
            alu_op: 0,
            run: true,
            tick_count: 0,
        }
    }

    pub fn opcode(&self) -> usize {
        ((self.cr >> 27) & 0x1F) as usize
    }

    pub fn mode(&self) -> usize {
        ((self.cr >> 25) & 0x3) as usize
    }

    pub fn alu_out(&self) -> i32 {
        let a = self.ac;
        let b = self.data_bus;
        match self.alu_op {
            0 => b,
            1 => a.wrapping_add(b),
            2 => a.wrapping_sub(b),
            3 => a.wrapping_mul(b),
            4 => a.wrapping_div(b),
            5 => a.wrapping_rem(b),
            6 => a & b,
            7 => a | b,
            8 => a ^ b,
            9 => a.wrapping_neg(),
            10 => !a,
            _ => 0,
        }
    }

    pub fn execute_microcommand(&mut self, micro: u32) {
        if micro & DR_TO_DATA_BUS != 0 {
            self.data_bus = self.dr;
        }
        if micro & IMM_TO_DATA_BUS != 0 {
            let imm = self.cr & 0x1FFFFFF;
            self.data_bus = ((imm as i32) << 7) >> 7;
        }
        if micro & ZERO_TO_DATA_BUS != 0 {
            self.data_bus = 0;
        }

        self.alu_op = ((micro >> 25) & 0xF) as u8;

        if micro & PC_TO_AR != 0 {
            self.ar = self.pc;
        }
        if micro & OPERAND_TO_AR != 0 {
            self.ar = self.cr & 0x1FFFFFF;
        }
        if micro & DR_TO_AR != 0 {
            self.ar = self.dr as u32;
        }
        if micro & SP_TO_AR != 0 {
            self.ar = self.sp;
        }

        if micro & AC_TO_DR != 0 {
            self.dr = self.ac;
        }
        if micro & PC_TO_DR != 0 {
            self.dr = self.pc as i32;
        }
        if micro & SP_TO_DR != 0 {
            self.dr = self.sp as i32;
        }

        if micro & ALU_TO_AC != 0 {
            self.ac = self.alu_out();
        }

        if micro & ALU_TO_PC != 0 {
            let cond = (micro >> 22) & 0x7;
            let ok = match cond {
                0 => true,
                1 => self.z,
                2 => !self.z,
                3 => self.n,
                4 => !self.n,
                _ => false,
            };
            if ok {
                self.pc = self.alu_out() as u32;
            }
        }

        if micro & DR_TO_AC != 0 {
            self.ac = self.dr;
        }
        if micro & DR_TO_CR != 0 {
            self.cr = self.dr as u32;
        }
        if micro & DR_TO_PC != 0 {
            self.pc = self.dr as u32;
        }

        if micro & PC_INCR != 0 {
            self.pc = self.pc.wrapping_add(1);
        }
        if micro & SP_INCR != 0 {
            self.sp = self.sp.wrapping_add(1);
        }
        if micro & SP_DECR != 0 {
            self.sp = self.sp.wrapping_sub(1);
        }

        if micro & UPDATE_FLAGS != 0 {
            let out = self.alu_out();
            self.z = out == 0;
            self.n = out < 0;
        }
    }
}
